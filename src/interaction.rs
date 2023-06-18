use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    core::{
        Controller, EntityRef, EntityRefBag, EntityRefSet, State, StateCommands, System,
        UpdateContext,
    },
    entity_insights::{EntityInsights, EntityLocation},
    equipment::{Equipment, EquipmentSlot},
    physics::CollisionState,
};

/// An interaction is represented as an actor, target pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Interaction {
    pub actor: EntityRef,
    pub target: EntityRef,
}

/// Denotes the conditions for start/end of interactions.
#[derive(Clone, Copy, Debug)]
pub enum InteractionType {
    /// Interaction can arbitrarily be started and only be explicitly ended.
    Whatevs,
    /// Interaction lasts only one frame.
    OneShot,
    /// Interaction can only be started if the actor and target touch.
    ContactRequired,
    /// Interaction can only be started if the actor and target touch. The interaction lasts only one frame.
    ContactRequiredOneShot,
}

impl InteractionType {
    /// Returns whether the interaction can start given the current state of the game.
    fn can_start(&self, actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        match self {
            InteractionType::OneShot => true,
            InteractionType::ContactRequired | InteractionType::ContactRequiredOneShot => {
                if let Some((target_coll_state,)) = state.select_one::<(CollisionState,)>(target) {
                    target_coll_state.colliding.contains(actor)
                } else {
                    false
                }
            }
            InteractionType::Whatevs => true,
        }
    }

    /// Returns whether a started interaction should end given the current state of the game.
    fn should_end(&self, actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        match self {
            InteractionType::Whatevs => false,
            InteractionType::OneShot => true,
            InteractionType::ContactRequiredOneShot => true,
            InteractionType::ContactRequired => !self.can_start(target, actor, state),
        }
    }
}

/// Denotes an interactable entity.
#[derive(Clone, Debug)]
pub struct Interactable {
    pub t: InteractionType,
    pub actors: EntityRefSet,
}

impl Interactable {
    pub fn new(intr_type: InteractionType) -> Self {
        Self {
            t: intr_type,
            actors: Default::default(),
        }
    }
}

/// A request to explicitly start an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryInteractReq(pub Interaction);

/// A request to explicitly end an interaction.
#[derive(Clone, Copy, Debug)]
pub struct TryUninteractReq(pub Interaction);

/// Emitted when an interaction is started.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InteractionStartedEvt(pub Interaction);

/// Emitted when an interaction has been ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InteractionEndedEvt(pub Interaction);

/// A system that handles interactions.
/// Starts or ends interactions and emits `InteractionStartedEvt` and `InteractionEndedEvt`.
/// Listens to `TryInteractReq` and `TryUninteractReq` events to explicitly start/end interactions.
/// Interactions can also be ended automatically on certain conditions.
#[derive(Clone, Debug, Default)]
pub struct InteractionSystem {
    interactions: HashSet<Interaction>,
}

impl System for InteractionSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Keep a set of events to emit at the end of the execution.
        let mut start_events = HashSet::<InteractionStartedEvt>::new();
        let mut end_events = HashSet::<InteractionEndedEvt>::new();
        // Get the invalidated actors that are still registered at the target.
        let valids = state.extract_validity_set();
        state
            .select::<(Interactable,)>()
            .for_each(|(target, (interactable,))| {
                let invalid_actors = interactable.actors.get_invalids(&valids).into_iter();
                end_events.extend(invalid_actors.map(|invalid_actor| {
                    InteractionEndedEvt(Interaction {
                        actor: invalid_actor,
                        target,
                    })
                }));
            });
        // Handle the current interactions that are invalid now.
        let auto_remove = self
            .interactions
            .iter()
            .filter(|intr| {
                if !valids.is_valid(&intr.actor) || !valids.is_valid(&intr.target) {
                    // Actor or target could be invalidated.
                    true
                } else if let Some((interactable,)) =
                    state.select_one::<(Interactable,)>(&intr.target)
                {
                    // Interaction may want to end itself.
                    interactable.t.should_end(&intr.actor, &intr.target, state)
                } else {
                    // If the target does not have an interactable component anymore, remove it as well.
                    true
                }
            })
            .cloned()
            .collect_vec();
        auto_remove.into_iter().for_each(|should_end| {
            if self.interactions.remove(&should_end) {
                end_events.insert(InteractionEndedEvt(should_end));
            }
        });
        // End all requested interactions.
        state.read_events::<TryUninteractReq>().for_each(|evt| {
            if self.interactions.remove(&evt.0) {
                end_events.insert(InteractionEndedEvt(evt.0));
            }
        });
        // Start new interactions.
        state.read_events::<TryInteractReq>().for_each(|evt| {
            if self.interactions.contains(&evt.0) {
                return;
            }
            if let Some((target,)) = state.select_one::<(Interactable,)>(&evt.0.target) {
                if target.t.can_start(&evt.0.actor, &evt.0.target, state) {
                    self.interactions.insert(evt.0);
                    start_events.insert(InteractionStartedEvt(evt.0));
                }
            }
        });
        // Emit the events & update the targets' state.
        end_events.into_iter().for_each(|evt| {
            cmds.emit_event(evt);
            cmds.update_component(&evt.0.target, move |interactable: &mut Interactable| {
                interactable.actors.try_remove(&evt.0.actor);
            })
        });
        start_events.into_iter().for_each(|evt| {
            cmds.emit_event(evt);
            cmds.update_component(&evt.0.target, move |interactable: &mut Interactable| {
                interactable.actors.insert(evt.0.actor);
            })
        });
    }
}

/// An actor that can interact with its surroundings.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractor;

/// A system that handles the entities that can interact with their surroundings.
#[derive(Clone, Debug, Default)]
pub struct ProximityInteractionSystem {
    /// Requested proximity interactions (actor -> target)
    potential_interactions: HashMap<EntityRef, EntityRef>,
}

impl System for ProximityInteractionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Try to end the proximity interaction with explicit key press.
        if ctx.control_map.end_interact_was_pressed {
            // End the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState)>()
                .for_each(|(e, _)| {
                    // Try to uninteract with the current target.
                    if let Some(current_target) = self.potential_interactions.remove(&e) {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: current_target,
                        }));
                    }
                });
        }
        // Try to start a proximity interaction.
        if ctx.control_map.start_interact_was_pressed {
            // Toggle the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState)>()
                .for_each(|(e, (_, coll_state))| {
                    // Try to uninteract with the current target.
                    if let Some(current_target) = self.potential_interactions.remove(&e) {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: current_target,
                        }));
                    }
                    // Find the first new interactable target that the entity is colliding with.
                    let interactable_target = coll_state.colliding.iter().find(|candidate| {
                        let is_interactable =
                            state.select_one::<(Interactable,)>(candidate).is_some();
                        let is_new = self
                            .potential_interactions
                            .get(&e)
                            .map(|curr_target| curr_target != *candidate)
                            .unwrap_or(true);
                        let is_on_ground = EntityInsights::of(&candidate, state).location
                            == EntityLocation::Ground;
                        is_interactable && is_new && is_on_ground
                    });
                    // Try to start the interaction with the new target.
                    if let Some(target_entity) = interactable_target {
                        self.potential_interactions.insert(e, *target_entity);
                        cmds.emit_event(TryInteractReq(Interaction {
                            actor: e,
                            target: *target_entity,
                        }));
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
            .select::<(HandInteractor, Equipment, Controller)>()
            .for_each(|(e, (_, equipment, _))| {
                // Get the left & right hand items of the hand interactor actor.
                let (lh_item, rh_item) = (
                    equipment.get(EquipmentSlot::LeftHand),
                    equipment.get(EquipmentSlot::RightHand),
                );
                // If left mouse is pressed, try to interact with the left hand item.
                if ctx.control_map.mouse_left_was_pressed {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryInteractReq(Interaction {
                            actor: e,
                            target: *lh_item,
                        }))
                    }
                }
                // If the left mouse is released, try to uninteract with the left hand item.
                if ctx.control_map.mouse_left_was_released {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: *lh_item,
                        }))
                    }
                }
                // If right mouse is pressed, try to interact with the right hand item.
                if ctx.control_map.mouse_right_was_pressed {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryInteractReq(Interaction {
                            actor: e,
                            target: *rh_item,
                        }))
                    }
                }
                // If the right mouse is released, try to uninteract with the right hand item.
                if ctx.control_map.mouse_right_was_released {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: *rh_item,
                        }))
                    }
                }
            })
    }
}
