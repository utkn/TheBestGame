use crate::{physics::ColliderInsights, prelude::*};

use super::{TryInteractReq, TryUninteractReq};

/// An actor that can be interacted by colliding entities by user input.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractable;

/// A system that handles the entities that can interact with their surroundings.
#[derive(Clone, Copy, Debug)]
pub struct ProximityInteractionSystem;

/// An event where the included actor tries to initiate a proximity interaction.
#[derive(Clone, Copy, Debug)]
pub struct StartProximityInteractReq(pub EntityRef);

/// An event where the included actor tries to end a proximity interaction.
#[derive(Clone, Copy, Debug)]
pub struct EndProximityInteractReq(pub EntityRef);

impl System for ProximityInteractionSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Try to start proximity interactions.
        state
            .read_events::<StartProximityInteractReq>()
            .for_each(|evt| {
                // Try to interact with all possible targets.
                let actor = evt.0;
                if let Some(contacts) = StateInsights::of(state).contacts_of(&actor) {
                    contacts
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|target| {
                            cmds.emit_event(TryInteractReq::new(actor, *target));
                        });
                }
            });
        // Try to uninteract with all possible targets.
        state
            .read_events::<EndProximityInteractReq>()
            .for_each(|evt| {
                let actor = evt.0;
                if let Some(contacts) = StateInsights::of(state).contacts_of(&actor) {
                    contacts
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|target| {
                            cmds.emit_event(TryUninteractReq::new(actor, *target));
                        });
                }
            });
    }
}
