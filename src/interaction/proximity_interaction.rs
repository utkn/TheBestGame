use crate::{
    physics::{CollisionEndEvt, CollisionState},
    prelude::*,
};

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
        state.read_events::<CollisionEndEvt>().for_each(|evt| {
            if let Some(_) = state.select_one::<(ProximityInteractable,)>(&evt.e1) {
                cmds.emit_event(TryUninteractReq::new(evt.e2, evt.e1));
            }
        });
        // Try to start proximity interactions.
        state
            .read_events::<StartProximityInteractReq>()
            .for_each(|evt| {
                let actor = evt.0;
                if let Some((actor_coll_state,)) = state.select_one::<(CollisionState,)>(&actor) {
                    // Try to uninteract with all possible targets.
                    actor_coll_state
                        .colliding
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|possible_target| {
                            cmds.emit_event(TryInteractReq::new(actor, *possible_target));
                        })
                }
            });
        state
            .read_events::<EndProximityInteractReq>()
            .for_each(|evt| {
                let actor = evt.0;
                if let Some((actor_coll_state,)) = state.select_one::<(CollisionState,)>(&actor) {
                    // Try to uninteract with all possible targets.
                    actor_coll_state
                        .colliding
                        .iter()
                        .filter(|e| state.select_one::<(ProximityInteractable,)>(e).is_some())
                        .for_each(|possible_target| {
                            cmds.emit_event(TryUninteractReq::new(actor, *possible_target));
                        })
                }
            });
    }
}
