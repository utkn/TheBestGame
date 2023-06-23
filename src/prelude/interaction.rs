use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use itertools::Itertools;
use rand::random;

mod interaction_acceptor;
mod interaction_delegate;

pub use interaction_acceptor::*;
pub use interaction_delegate::*;

use crate::prelude::*;

/// Represents an interaction that can occur between two entities in the game.
pub trait Interaction: 'static + std::fmt::Debug + Clone {
    fn priority() -> usize;
    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool;
    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool;
    fn can_end_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool;
    fn can_end_targeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
    fn interaction_exists(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        if let Some((target_intr,)) = state.select_one::<(InteractTarget<Self>,)>(target) {
            target_intr.actors.contains(actor)
        } else {
            false
        }
    }
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
    pub consensus_id: usize,
    pub actor: EntityRef,
    pub target: EntityRef,
}
impl TryUninteractReq {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            consensus_id: random(),
            actor,
            target,
        }
    }
}

/// A request to explicitly start an interaction.
#[derive(Clone, Copy, Debug)]
pub struct InteractReq<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> InteractReq<I> {
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
pub struct UninteractReq<I: Interaction> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: Interaction> UninteractReq<I> {
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
    fn interaction_exists(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        I::interaction_exists(actor, target, state)
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        actor != target
            && if let Some(_) = state.select_one::<(InteractTarget<I>,)>(target) {
                !Self::interaction_exists(actor, target, state)
                    && I::can_start_untargeted(actor, target, state)
            } else {
                false
            }
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        actor != target
            && if let Some(_) = state.select_one::<(InteractTarget<I>,)>(target) {
                !Self::interaction_exists(actor, target, state)
                    && I::can_start_targeted(actor, target, state)
            } else {
                false
            }
    }

    fn can_end_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::interaction_exists(actor, target, state)
            && I::can_end_untargeted(actor, target, state)
    }

    fn can_end_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::interaction_exists(actor, target, state) && I::can_end_targeted(actor, target, state)
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
        let to_end: HashSet<_> = invalidated_interactions
            // End the accepted uninteract proposals.
            .chain(
                state
                    .read_events::<UninteractAcceptedEvt>()
                    .filter(|evt| evt.proposer_tid == TypeId::of::<I>())
                    .map(|evt| (evt.actor, evt.target))
                    .filter(|(actor, target)| Self::can_end_untargeted(&actor, &target, state)),
            )
            // Bypass the consensus for targeted uninteracts.
            .chain(
                state
                    .read_events::<UninteractReq<I>>()
                    .map(|evt| (evt.actor, evt.target))
                    .filter(|(actor, target)| Self::can_end_targeted(actor, target, state)),
            )
            .collect();
        to_end.into_iter().for_each(|(actor, target)| {
            if Self::interaction_exists(&actor, &target, state) {
                println!("ending {:?} -> {:?}: {:?}", actor, target, self.pd);
                cmds.emit_event(InteractionEndedEvt::<I>::new(actor, target));
                cmds.update_component(&target, move |interactable: &mut InteractTarget<I>| {
                    interactable.actors.try_remove(&actor);
                });
            }
        });
        let to_start: HashSet<_> = state
            .read_events::<InteractAcceptedEvt>()
            .filter(|evt| evt.proposer_tid == TypeId::of::<I>())
            .map(|evt| (evt.actor, evt.target))
            .filter(|(actor, target)| Self::can_start_untargeted(&actor, &target, state))
            .chain(
                // Bypass the consensus for targeted interaction requests
                state
                    .read_events::<InteractReq<I>>()
                    .map(|evt| (evt.actor, evt.target))
                    .filter(|(actor, target)| Self::can_start_targeted(actor, target, state)),
            )
            .filter(|(actor, target)| !Self::interaction_exists(actor, target, state))
            .collect();
        to_start.into_iter().for_each(|(actor, target)| {
            cmds.emit_event(InteractionStartedEvt::<I>::new(actor, target));
            println!("starting {:?} -> {:?}: {:?}", actor, target, self.pd);
            cmds.update_component(&target, move |interactable: &mut InteractTarget<I>| {
                interactable.actors.insert(actor);
            });
        });
        // Propose interactions in response to untargeted interact/uninteract requests.
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if Self::can_start_untargeted(&evt.actor, &evt.target, state) {
                // Otherwise, propose normally.
                cmds.emit_event(ProposeInteractEvt::from_req::<I>(evt));
            };
        });
        state.read_events::<TryUninteractReq>().for_each(|evt| {
            if Self::can_end_untargeted(&evt.actor, &evt.target, state) {
                // Otherwise, propose normally.
                cmds.emit_event(ProposeUninteractEvt::from_req::<I>(evt));
            };
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct ProposeInteractEvt {
    proposer_id: std::any::TypeId,
    consensus_id: usize,
    priority: usize,
    actor: EntityRef,
    target: EntityRef,
}

impl ProposeInteractEvt {
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
struct ProposeUninteractEvt {
    proposer_id: std::any::TypeId,
    consensus_id: usize,
    priority: usize,
    actor: EntityRef,
    target: EntityRef,
}

impl ProposeUninteractEvt {
    /// Creates a new proposal in response to the given [`TryInteractReq`].
    fn from_req<I: Interaction>(req: &TryUninteractReq) -> Self {
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
