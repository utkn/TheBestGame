use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use itertools::Itertools;
use rand::random;

mod hand_interaction;
mod interaction_acceptor;
mod interaction_delegate;
mod proximity_interaction;

pub use hand_interaction::*;
pub use interaction_acceptor::*;
pub use interaction_delegate::*;
pub use proximity_interaction::*;

use crate::prelude::*;

/// Represents an interaction that can occur between two entities in the game.
pub trait Interaction: 'static + std::fmt::Debug + Clone {
    fn priority() -> usize;
    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool;
}

/// Denotes an interactable entity as the target of the interaction `I`.
#[derive(Clone, Debug)]
pub struct InteractTarget<I: Interaction> {
    pub actors: EntityRefSet,
    pd: PhantomData<I>,
}

impl<I: Interaction> Default for InteractTarget<I> {
    fn default() -> Self {
        Self {
            actors: Default::default(),
            pd: Default::default(),
        }
    }
}

/// A request to explicitly start an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryInteractReq {
    pub consensus_id: usize,
    pub actor: EntityRef,
    pub target: EntityRef,
}

impl TryInteractReq {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            consensus_id: random(),
            actor,
            target,
        }
    }
}

/// A request to explicitly end an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryUninteractReq {
    pub actor: EntityRef,
    pub target: EntityRef,
}
impl TryUninteractReq {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self { actor, target }
    }
}

/// A request to explicitly start an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryInteractTargetedReq<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> TryInteractTargetedReq<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: PhantomData::default(),
        }
    }
}

/// A request to explicitly end an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryUninteractTargetedReq<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> TryUninteractTargetedReq<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: PhantomData::default(),
        }
    }
}

/// A system that proposes interactions defined by `I` in response to [`TryInteractReq`]s.
#[derive(Clone, Debug)]
pub struct InteractionSystem<I: Interaction> {
    pd: PhantomData<I>,
}

impl<I: Interaction> Default for InteractionSystem<I> {
    fn default() -> Self {
        Self {
            pd: Default::default(),
        }
    }
}

impl<I: Interaction> InteractionSystem<I> {
    pub fn interaction_exists(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        if let Some((intr,)) = state.select_one::<(InteractTarget<I>,)>(target) {
            intr.actors.contains(actor)
        } else {
            false
        }
    }

    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        if let Some(_) = state.select_one::<(InteractTarget<I>,)>(target) {
            !Self::interaction_exists(actor, target, state) && I::can_start(actor, target, state)
        } else {
            false
        }
    }

    fn interactions<'a>(state: &'a State) -> impl Iterator<Item = (EntityRef, EntityRef)> + 'a {
        state
            .select::<(InteractTarget<I>,)>()
            .flat_map(|(target, (intr,))| {
                intr.actors
                    .iter()
                    .map(|actor| (*actor, target))
                    .collect_vec()
            })
    }
}

impl<I: Interaction> System for InteractionSystem<I> {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Auto invalidate interactions.
        let invalidated_interactions = Self::interactions(state).filter(|(actor, target)| {
            state.will_be_removed(actor) || state.will_be_removed(target)
        });
        // End interactions in response to explicit uninteract requests as well.
        let to_end: HashSet<_> = state
            .read_events::<TryUninteractReq>()
            .map(|evt| (evt.actor, evt.target))
            .chain(
                state
                    .read_events::<TryUninteractTargetedReq<I>>()
                    .map(|evt| (evt.actor, evt.target)),
            )
            .chain(invalidated_interactions)
            .collect();
        to_end.into_iter().for_each(|(actor, target)| {
            if Self::interaction_exists(&actor, &target, state) {
                // println!("ending {:?} -> {:?}: {:?}", actor, target, self.pd);
                cmds.emit_event(InteractionEndedEvt::<I>::new(actor, target));
                cmds.update_component(&target, move |interactable: &mut InteractTarget<I>| {
                    interactable.actors.try_remove(&actor);
                });
            }
        });
        // Propose interactions in response to try interaction requests.
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if Self::can_start(&evt.actor, &evt.target, state) {
                // Otherwise, propose normally.
                cmds.emit_event(ProposeInteractionEvt::from_req::<I>(evt));
            };
        });
        let to_start: HashSet<_> = state
            .read_events::<InteractionAcceptedEvt>()
            .filter(|evt| evt.proposer_tid == TypeId::of::<I>())
            .map(|evt| (evt.actor, evt.target))
            .chain(
                // Bypass the consensus for targeted interaction requests
                state
                    .read_events::<TryInteractTargetedReq<I>>()
                    .map(|evt| (evt.actor, evt.target)),
            )
            .filter(|(actor, target)| Self::can_start(&actor, &target, state))
            .collect();
        to_start.into_iter().for_each(|(actor, target)| {
            cmds.emit_event(InteractionStartedEvt::<I>::new(actor, target));
            // println!("starting {:?} -> {:?}: {:?}", actor, target, self.pd);
            cmds.update_component(&target, move |interactable: &mut InteractTarget<I>| {
                interactable.actors.insert(actor);
            });
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct ProposeInteractionEvt {
    proposer_id: std::any::TypeId,
    consensus_id: usize,
    priority: usize,
    actor: EntityRef,
    target: EntityRef,
}

impl ProposeInteractionEvt {
    /// Creates a new proposal in response to the given [`TryInteractReq`].
    fn from_req<I: Interaction>(req: &TryInteractReq) -> Self {
        Self {
            consensus_id: req.consensus_id,
            proposer_id: std::any::TypeId::of::<I>(),
            priority: I::priority(),
            actor: req.actor,
            target: req.target,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionStartedEvt<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> InteractionStartedEvt<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionEndedEvt<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> InteractionEndedEvt<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: Default::default(),
        }
    }
}
