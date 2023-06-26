use crate::{
    character::CharacterInsights,
    item::{ItemInsights, ItemLocation},
    physics::ColliderInsights,
    prelude::*,
};

use super::{Storage, StorageBundle};

/// A [`Storage`] can act as an activation/unactivation [`Interaction`].
impl Interaction for Storage {
    fn priority() -> usize {
        50
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        state.select_one::<(Storage,)>(target).is_some()
            && StateInsights::of(state).is_character(actor)
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
            && StateInsights::of(state).location_of(target) == ItemLocation::Ground
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StorageDeactivationSystem;

impl System for StorageDeactivationSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Storage, InteractTarget<Storage>)>()
            .for_each(|(storage_entity, _)| {
                if let Some(storage_bundle) = state.read_bundle::<StorageBundle>(&storage_entity) {
                    StateInsights::of(state)
                        .new_collision_enders_of(&storage_bundle.activator)
                        .into_iter()
                        .for_each(|actor| {
                            cmds.emit_event(UninteractReq::<Storage>::new(*actor, storage_entity));
                        });
                }
            });
    }
}
