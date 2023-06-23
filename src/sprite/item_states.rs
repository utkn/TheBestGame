use crate::{
    item::{EquipmentSlot, ItemInsights, ItemLocation},
    prelude::StateInsights,
};

use super::{EntityStateGraph, TagGenerator};

pub const ITEM_ON_GROUND: TagGenerator = TagGenerator {
    tag: "on_ground",
    is_state_of: |e, state| StateInsights::of(state).location_of(e) == ItemLocation::Ground,
};

pub const ITEM_IN_STORAGE: TagGenerator = TagGenerator {
    tag: "in_storage",
    is_state_of: |e, state| {
        matches!(
            StateInsights::of(state).location_of(e,),
            ItemLocation::Storage(_)
        )
    },
};

pub const ITEM_IN_EQUIPMENT: TagGenerator = TagGenerator {
    tag: "in_equipment",
    is_state_of: |e, state| {
        matches!(
            StateInsights::of(state).location_of(e,),
            ItemLocation::Equipment(_)
        )
    },
};

pub const ITEM_ON_HEAD: TagGenerator = TagGenerator {
    tag: "on_head",
    is_state_of: |e, state| {
        if let Some(slots) = StateInsights::of(state).equipped_slots_of(e) {
            slots.contains(&EquipmentSlot::Head)
        } else {
            false
        }
    },
};

pub const ITEM_ON_TORSO: TagGenerator = TagGenerator {
    tag: "on_torso",
    is_state_of: |e, state| {
        if let Some(slots) = StateInsights::of(state).equipped_slots_of(e) {
            slots.contains(&EquipmentSlot::Torso)
        } else {
            false
        }
    },
};

pub const ITEM_ON_LEGS: TagGenerator = TagGenerator {
    tag: "on_legs",
    is_state_of: |e, state| {
        if let Some(slots) = StateInsights::of(state).equipped_slots_of(e) {
            slots.contains(&EquipmentSlot::Legs)
        } else {
            false
        }
    },
};

pub const ITEM_ON_HAND: TagGenerator = TagGenerator {
    tag: "on_hand",
    is_state_of: |e, state| {
        if let Some(slots) = StateInsights::of(state).equipped_slots_of(e) {
            slots.contains(&EquipmentSlot::LeftHand) || slots.contains(&EquipmentSlot::RightHand)
        } else {
            false
        }
    },
};

pub const ITEM_ON_FEET: TagGenerator = TagGenerator {
    tag: "on_hand",
    is_state_of: |e, state| {
        if let Some(slots) = StateInsights::of(state).equipped_slots_of(e) {
            slots.contains(&EquipmentSlot::Feet)
        } else {
            false
        }
    },
};

pub const ITEM_STATE_GRAPH: EntityStateGraph = EntityStateGraph(
    "item",
    &[
        &[ITEM_ON_GROUND],
        &[ITEM_ON_GROUND, ITEM_IN_STORAGE],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT, ITEM_ON_HEAD],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT, ITEM_ON_HAND],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT, ITEM_ON_TORSO],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT, ITEM_ON_LEGS],
        &[ITEM_ON_GROUND, ITEM_IN_EQUIPMENT, ITEM_ON_FEET],
    ],
);
