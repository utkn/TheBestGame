use crate::{character::CharacterInsights, item::Storage, prelude::*};

use super::{Vehicle, VehicleInsights};

impl Interaction for Vehicle {
    fn priority() -> usize {
        Storage::priority() + 10
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let insights = StateInsights::of(state);
        insights.is_vehicle(target) && insights.is_character(actor)
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}
