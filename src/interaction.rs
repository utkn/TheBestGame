use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    core::{
        Controller, EntityRef, EntityRefBag, EntityRefSet, State, StateCommands, System,
        UpdateContext,
    },
    equipment::{Equipment, EquipmentSlot},
    physics::CollisionState,
};

/// An interaction is represented as an actor, target pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Interaction {
    pub actor: EntityRef,
    pub target: EntityRef,
}

/// Whether
#[derive(Clone, Copy, Debug)]
pub enum InteractionType {
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
        }
    }

    /// Returns whether a started interaction should end given the current state of the game.
    fn should_end(&self, actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        match self {
            InteractionType::OneShot => true,
            InteractionType::ContactRequiredOneShot => true,
            InteractionType::ContactRequired => !self.can_start(target, actor, state),
        }
    }
}

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

#[derive(Clone, Copy, Debug)]
pub struct TryInteractReq(pub Interaction);

#[derive(Clone, Copy, Debug)]
pub struct TryUninteractReq(pub Interaction);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InteractionStartedEvt(pub Interaction);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InteractionEndedEvt(pub Interaction);

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
#[derive(Clone, Copy, Default, Debug)]
pub struct ProximityInteractor {
    pub target: Option<EntityRef>,
}

#[derive(Clone, Debug)]
pub struct ProximityInteractionSystem;

impl System for ProximityInteractionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Remove the proximity interactions if they were killed.
        state.read_events::<InteractionEndedEvt>().for_each(|evt| {
            // Get the actor's proximity target.
            let actor_proximity_target = state
                .select_one::<(ProximityInteractor, Controller)>(&evt.0.actor)
                .map(|(proximity_interactor, _)| proximity_interactor.target)
                .flatten();
            // If it is identical to the target of this ended interaction, remove it from the proximity target.
            if let Some(actor_proximity_target) = actor_proximity_target {
                if actor_proximity_target == evt.0.target {
                    cmds.update_component(&evt.0.actor, move |pi: &mut ProximityInteractor| {
                        pi.target = None;
                    })
                }
            }
        });
        // Handle the new interactions that should be handled as a proximity interaction.
        state
            .read_events::<InteractionStartedEvt>()
            .for_each(|evt| {
                // If the actor is a proximity interactor...
                if let Some((_, actor_coll_state, _)) =
                    state.select_one::<(ProximityInteractor, CollisionState, Controller)>(
                        &evt.0.actor,
                    )
                {
                    // ... and it collides with the target of this interaction...
                    if actor_coll_state.colliding.contains(&evt.0.target) {
                        // ... then set the proximity target of the actor.
                        let target = evt.0.target;
                        cmds.update_component(&evt.0.actor, move |pi: &mut ProximityInteractor| {
                            pi.target = Some(target);
                        });
                    }
                }
            });
        // End or toggle the proximity interaction, depending on the user input.
        if ctx.control_map.end_interact_was_pressed {
            // End the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState, Controller)>()
                .for_each(|(e, (pi, _, _))| {
                    // Try to uninteract with the current target.
                    if let Some(current_target) = pi.target {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: current_target,
                        }));
                    }
                });
        } else if ctx.control_map.start_interact_was_pressed {
            // Toggle the proximity interaction.
            state
                .select::<(ProximityInteractor, CollisionState, Controller)>()
                .for_each(|(e, (pi, coll_state, _))| {
                    // Try to uninteract with the current target.
                    if let Some(current_target) = pi.target {
                        cmds.emit_event(TryUninteractReq(Interaction {
                            actor: e,
                            target: current_target,
                        }));
                    }
                    // Find the first new interactable target that the entity is colliding with.
                    let interactable_target = coll_state.colliding.iter().find(|candidate| {
                        let is_interactable =
                            state.select_one::<(Interactable,)>(candidate).is_some();
                        let is_new = pi
                            .target
                            .map(|curr_target| curr_target != **candidate)
                            .unwrap_or(true);
                        is_interactable && is_new
                    });
                    // Try to start the interaction with the new target.
                    if let Some(target_entity) = interactable_target {
                        cmds.emit_event(TryInteractReq(Interaction {
                            actor: e,
                            target: *target_entity,
                        }));
                    }
                });
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HandInteractor;

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
                // If right mouse is pressed, try to interact with the right hand item.
                if ctx.control_map.mouse_right_was_pressed {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryInteractReq(Interaction {
                            actor: e,
                            target: *rh_item,
                        }))
                    }
                }
            })
    }
}
