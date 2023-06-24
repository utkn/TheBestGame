use crate::{
    character::CharacterInsights,
    item::{ItemInsights, ItemLocation, Storage},
    prelude::*,
};

use super::{Equipment, EquipmentInsights};

/// An [`Equipment`] can act as an activation/unactivation [`Interaction`].
impl Interaction for Equipment {
    fn priority() -> usize {
        Storage::priority()
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let insights = StateInsights::of(state);
        insights.has_equipment(target) && insights.is_character(actor)
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
            && StateInsights::of(state).location_of(target) == ItemLocation::Ground
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}
