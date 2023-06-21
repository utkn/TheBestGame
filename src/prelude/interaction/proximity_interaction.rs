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
        // Try to end the proximity interaction with collision end events.
        state
            .select::<(ProximityInteractable,)>()
            .for_each(|(target, _)| {
                EntityInsights::of(&target, state)
                    .new_collision_enders()
                    .into_iter()
                    .for_each(|actor| {
                        cmds.emit_event(TryUninteractReq::new(actor, target));
                    });
            });
        // Try to start proximity interactions.
        state
            .read_events::<StartProximityInteractReq>()
            .for_each(|evt| {
                // Try to interact with all possible targets.
                let actor = evt.0;
                EntityInsights::of(&actor, state)
                    .contacts()
                    .iter()
                    .for_each(|target| {
                        cmds.emit_event(TryInteractReq::new(actor, *target));
                    });
            });
        state
            .read_events::<EndProximityInteractReq>()
            .for_each(|evt| {
                // Try to uninteract with all possible targets.
                let actor = evt.0;
                EntityInsights::of(&actor, state)
                    .contacts()
                    .iter()
                    .for_each(|target| {
                        cmds.emit_event(TryUninteractReq::new(actor, *target));
                    });
            });
    }
}
