use crate::{
    character::CharacterInsights,
    item::{ItemInsights, ItemLocation},
    prelude::*,
};

use super::Storage;

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
