use std::{any::TypeId, marker::PhantomData};

use itertools::Itertools;
use rand::random;

use crate::{
    core::*,
    equipment::{Equipment, EquipmentSlot},
    physics::CollisionState,
};

#[allow(unused_variables)]
pub trait InteractionType: 'static + std::fmt::Debug + Clone {
    fn priority() -> usize;
    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool;
}

/// Denotes an interactable entity.
#[derive(Clone, Debug)]
pub struct Interactable<I: InteractionType> {
    pub actors: EntityRefSet,
    pd: PhantomData<I>,
}

impl<I: InteractionType> Default for Interactable<I> {
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

/// A request to explicitly end an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryUninteractTargetedReq<I: InteractionType> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: InteractionType> TryUninteractTargetedReq<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: PhantomData::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProposeInteractionEvt {
    proposer_id: std::any::TypeId,
    consensus_id: usize,
    priority: usize,
    actor: EntityRef,
    target: EntityRef,
}

impl ProposeInteractionEvt {
    /// Creates a new proposal in response to the given [`TryInteractReq`].
    pub fn from_req<I: InteractionType>(req: &TryInteractReq) -> Self {
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
pub struct InteractionAcceptedEvt {
    consensus_id: usize,
    /// The unique type id of the proposer. Used to distinguish between different proposals for the same request.
    proposer_tid: std::any::TypeId,
    actor: EntityRef,
    target: EntityRef,
}

impl From<&ProposeInteractionEvt> for InteractionAcceptedEvt {
    /// Constructs from the given proposal.
    fn from(proposal: &ProposeInteractionEvt) -> Self {
        InteractionAcceptedEvt {
            consensus_id: proposal.consensus_id,
            proposer_tid: proposal.proposer_id,
            actor: proposal.actor,
            target: proposal.target,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionStartedEvt<I: InteractionType> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: InteractionType> InteractionStartedEvt<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionEndedEvt<I: InteractionType> {
    pub actor: EntityRef,
    pub target: EntityRef,
    pd: PhantomData<I>,
}

impl<I: InteractionType> InteractionEndedEvt<I> {
    pub fn new(actor: EntityRef, target: EntityRef) -> Self {
        Self {
            actor,
            target,
            pd: Default::default(),
        }
    }
}

pub struct InteractionAcceptorSystem;

impl System for InteractionAcceptorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        let consensus_instances = state
            .read_events::<ProposeInteractionEvt>()
            .into_group_map_by(|evt| evt.consensus_id);
        consensus_instances.into_iter().for_each(|(_, proposals)| {
            let picked_proposal = proposals
                .into_iter()
                .max_by_key(|proposal| proposal.priority);
            if let Some(picked_proposal) = picked_proposal {
                cmds.emit_event(InteractionAcceptedEvt::from(picked_proposal));
            }
        });
    }
}

/// A system that proposes interactions defined by `I` in response to [`TryInteractReq`]s.
#[derive(Clone, Debug)]
pub struct InteractionSystem<I: InteractionType> {
    pd: PhantomData<I>,
}

impl<I: InteractionType> Default for InteractionSystem<I> {
    fn default() -> Self {
        Self {
            pd: Default::default(),
        }
    }
}

impl<I: InteractionType> InteractionSystem<I> {
    pub fn interaction_exists(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        if let Some((intr,)) = state.select_one::<(Interactable<I>,)>(target) {
            intr.actors.contains(actor)
        } else {
            false
        }
    }

    pub fn interactions<'a>(state: &'a State) -> impl Iterator<Item = (EntityRef, EntityRef)> + 'a {
        state
            .select::<(Interactable<I>,)>()
            .flat_map(|(target, (intr,))| {
                intr.actors
                    .iter()
                    .map(|actor| (*actor, target))
                    .collect_vec()
            })
    }
}

impl<I: InteractionType> System for InteractionSystem<I> {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Auto invalidate interactions.
        let invalidated_interactions = Self::interactions(state)
            .filter(|(actor, target)| state.will_be_removed(actor) || state.will_be_removed(target))
            .collect_vec();
        invalidated_interactions
            .into_iter()
            .for_each(|(actor, target)| {
                cmds.emit_event(InteractionEndedEvt::<I>::new(actor, target));
                cmds.update_component(&target, move |interactable: &mut Interactable<I>| {
                    interactable.actors.try_remove(&actor);
                });
            });
        // End interactions in response to explicit uninteract requests.
        let uninteract_requests = state
            .read_events::<TryUninteractTargetedReq<I>>()
            .map(|evt| (evt.actor, evt.target))
            .chain(
                state
                    .read_events::<TryUninteractReq>()
                    .map(|evt| (evt.actor, evt.target)),
            );
        uninteract_requests.for_each(|(actor, target)| {
            if Self::interaction_exists(&actor, &target, state) {
                cmds.emit_event(InteractionEndedEvt::<I>::new(actor, target));
                cmds.update_component(&target, move |interactable: &mut Interactable<I>| {
                    interactable.actors.try_remove(&actor);
                });
            }
        });
        // Propose interactions in response to try interaction requests.
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if !Self::interaction_exists(&evt.actor, &evt.target, state)
                && I::can_start(&evt.actor, &evt.target, state)
            {
                // Otherwise, propose normally.
                cmds.emit_event(ProposeInteractionEvt::from_req::<I>(evt));
            };
        });
        // Check if our proposal was accepted.
        state
            .read_events::<InteractionAcceptedEvt>()
            .for_each(|evt| {
                if !Self::interaction_exists(&evt.actor, &evt.target, state)
                    && evt.proposer_tid == TypeId::of::<I>()
                {
                    let (actor, target) = (evt.actor, evt.target);
                    cmds.emit_event(InteractionStartedEvt::<I>::new(actor, target));
                    cmds.update_component(&target, move |interactable: &mut Interactable<I>| {
                        interactable.actors.insert(actor);
                    });
                }
            });
    }
}

/// An actor that can interact with its surroundings.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractor;

/// An actor that can be interacted by colliding entities.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractable;

/// A system that handles the entities that can interact with their surroundings.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractionSystem;

impl System for ProximityInteractionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Try to end the proximity interaction with explicit key press.
        if ctx.control_map.end_interact_was_pressed {
            // End the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState)>()
                .for_each(|(e, (_, coll_state))| {
                    // Try to uninteract with all possible targets.
                    coll_state
                        .colliding
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|possible_target| {
                            cmds.emit_event(TryUninteractReq {
                                actor: e,
                                target: *possible_target,
                            });
                        })
                });
        }
        // Try to start a proximity interaction.
        if ctx.control_map.start_interact_was_pressed {
            // Toggle the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState)>()
                .for_each(|(e, (_, coll_state))| {
                    // Try to interact with all possible targets.
                    coll_state
                        .colliding
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|candidate| {
                            cmds.emit_event(TryInteractReq::new(e, *candidate));
                        });
                });
        }
    }
}

/// An actor that can interact with what they have on their hands (i.e., in their appropriate equipment slot).
#[derive(Clone, Copy, Debug)]
pub struct HandInteractor;

/// A system that handles the entities that can interact with their equipment.
#[derive(Clone, Copy, Debug)]
pub struct HandInteractionSystem;

impl System for HandInteractionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(HandInteractor, Equipment)>()
            .for_each(|(e, (_, equipment))| {
                // Get the left & right hand items of the hand interactor actor.
                let (lh_item, rh_item) = (
                    equipment.get(EquipmentSlot::LeftHand),
                    equipment.get(EquipmentSlot::RightHand),
                );
                // If left mouse is pressed, try to interact with the left hand item.
                if ctx.control_map.mouse_left_was_pressed {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryInteractReq::new(e, *lh_item));
                    }
                }
                if ctx.control_map.mouse_left_was_released {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryUninteractReq::new(e, *lh_item));
                    }
                }
                // If right mouse is pressed, try to interact with the right hand item.
                if ctx.control_map.mouse_right_was_pressed {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryInteractReq::new(e, *rh_item));
                    }
                }
                if ctx.control_map.mouse_right_was_released {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryUninteractReq::new(e, *rh_item));
                    }
                }
            })
    }
}

/// A component that converts interaction requests to its parent while acting as a target.
#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegate(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegateSystem;

impl System for InteractionDelegateSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(InteractionDelegate,)>()
            .for_each(|(delegee, (delegate,))| {
                if !state.is_valid(&delegate.0) {
                    cmds.remove_component::<InteractionDelegate>(&delegee);
                }
            });
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(InteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryInteractReq::new(evt.actor, target_delegate.0));
            }
        });
    }
}
