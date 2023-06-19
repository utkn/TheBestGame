use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use itertools::Itertools;
use rand::random;

use crate::{
    core::*,
    entity_insights::{EntityInsights, EntityLocation},
    equipment::{Equipment, EquipmentSlot},
    physics::CollisionState,
};

/// Denotes an interactable entity.
#[derive(Clone, Debug, Default)]
pub struct Interactable {
    pub actors: EntityRefSet,
}

#[allow(unused_variables)]
pub trait InteractionType: 'static + std::fmt::Debug + Clone {
    fn priority() -> usize {
        0
    }

    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::valid_actors(target, state)
            .map(|actors| actors.contains(actor))
            .unwrap_or(false)
    }

    fn should_end(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::valid_actors(target, state)
            .map(|actors| !actors.contains(actor))
            .unwrap_or(true)
    }

    fn valid_actors(target: &EntityRef, state: &State) -> Option<HashSet<EntityRef>>;
    fn on_start(actor: &EntityRef, target: &EntityRef, state: &State, cmds: &mut StateCommands) {}
    fn on_end(actor: &EntityRef, target: &EntityRef, state: &State, cmds: &mut StateCommands) {}
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

    /// Creates a new proposal in response to the given [`TryInteractReq`] which will always succeed.
    pub fn ensurement_from_req<I: InteractionType>(req: &TryInteractReq) -> Self {
        let mut evt = Self::from_req::<I>(req);
        evt.priority = usize::MAX;
        evt
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
pub struct InteractionProposerSystem<I: InteractionType> {
    interactions: HashSet<(EntityRef, EntityRef)>,
    pd: PhantomData<I>,
}

impl<I: InteractionType> Default for InteractionProposerSystem<I> {
    fn default() -> Self {
        Self {
            interactions: Default::default(),
            pd: Default::default(),
        }
    }
}

impl<I: InteractionType> System for InteractionProposerSystem<I> {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Auto invalidate interactions.
        let invalidated_interactions = self
            .interactions
            .iter()
            .filter(|(actor, target)| {
                I::should_end(actor, target, state)
                    || !state.is_valid(actor)
                    || !state.is_valid(target)
            })
            .cloned()
            .collect_vec();
        invalidated_interactions
            .into_iter()
            .for_each(|(actor, target)| {
                self.interactions.remove(&(actor, target));
                I::on_end(&actor, &target, state, cmds);
                cmds.update_component(&target, move |interactable: &mut Interactable| {
                    interactable.actors.try_remove(&actor);
                });
            });
        // End interactions in response to explicit uninteract requests.
        state.read_events::<TryUninteractReq>().for_each(|evt| {
            if self.interactions.remove(&(evt.actor, evt.target)) {
                let (actor, target) = (evt.actor, evt.target);
                I::on_end(&actor, &target, state, cmds);
                cmds.update_component(&target, move |interactable: &mut Interactable| {
                    interactable.actors.try_remove(&actor);
                });
            }
        });
        // Propose interactions in response to try interaction requests.
        state.read_events::<TryInteractReq>().for_each(|evt| {
            // If we already have this interaction, no other interactions should start. We must ensure
            // that the acceptor will accept our proposal. Hence, we emit an `ensurement`.
            if self.interactions.contains(&(evt.actor, evt.target)) {
                cmds.emit_event(ProposeInteractionEvt::ensurement_from_req::<I>(evt));
            } else if !self.interactions.contains(&(evt.actor, evt.target))
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
                if !self.interactions.contains(&(evt.actor, evt.target))
                    && evt.proposer_tid == TypeId::of::<I>()
                {
                    let (actor, target) = (evt.actor, evt.target);
                    self.interactions.insert((actor, target));
                    I::on_start(&actor, &target, state, cmds);
                    cmds.update_component(&target, move |interactable: &mut Interactable| {
                        interactable.actors.insert(actor);
                    });
                }
            });
    }
}

/// An actor that can interact with its surroundings.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractor;

/// A system that handles the entities that can interact with their surroundings.
#[derive(Clone, Debug, Default)]
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
                    coll_state.colliding.iter().for_each(|possible_target| {
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
                    // Find the first new interactable target that the entity is colliding with.
                    let interactable_target = coll_state.colliding.iter().find(|candidate| {
                        let is_interactable =
                            state.select_one::<(Interactable,)>(candidate).is_some();
                        let is_new = state
                            .select_one::<(Interactable,)>(candidate)
                            .map(|(candidate_interactable,)| {
                                !candidate_interactable.actors.contains(&e)
                            })
                            .unwrap_or(false);
                        let is_on_ground = EntityInsights::of(&candidate, state).location
                            == EntityLocation::Ground;
                        is_interactable && is_new && is_on_ground
                    });
                    // Try to start the interaction with the new target.
                    if let Some(target_entity) = interactable_target {
                        cmds.emit_event(TryInteractReq::new(e, *target_entity));
                    }
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
                // If the left mouse is released, try to uninteract with the left hand item.
                if ctx.control_map.mouse_left_was_released {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryUninteractReq {
                            actor: e,
                            target: *lh_item,
                        });
                    }
                }
                // If right mouse is pressed, try to interact with the right hand item.
                if ctx.control_map.mouse_right_was_pressed {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryInteractReq::new(e, *rh_item));
                    }
                }
                // If the right mouse is released, try to uninteract with the right hand item.
                if ctx.control_map.mouse_right_was_released {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryUninteractReq {
                            actor: e,
                            target: *rh_item,
                        });
                    }
                }
            })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegate(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct InteractionDelegateSystem;

impl System for InteractionDelegateSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if let Some((target_delegate,)) =
                state.select_one::<(InteractionDelegate,)>(&evt.target)
            {
                cmds.emit_event(TryInteractReq::new(evt.actor, target_delegate.0));
            }
        });
    }
}
