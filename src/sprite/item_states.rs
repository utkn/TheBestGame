use crate::{
    item::{EquipmentSlot, ItemInsights, ItemLocation},
    prelude::EntityInsights,
};

use super::{EntityState, EntityStateGraph};

pub const ITEM_ON_GROUND: EntityState = EntityState {
    tag: "on_ground",
    is_state_of: |e, state| EntityInsights::of(e, state).location() == ItemLocation::Ground,
};

pub const ITEM_IN_STORAGE: EntityState = EntityState {
    tag: "in_storage",
    is_state_of: |e, state| {
        matches!(
            EntityInsights::of(e, state).location(),
            ItemLocation::Storage(_)
        )
    },
};

pub const ITEM_IN_EQUIPMENT: EntityState = EntityState {
    tag: "in_equipment",
    is_state_of: |e, state| {
        matches!(
            EntityInsights::of(e, state).location(),
            ItemLocation::Equipment(_)
        )
    },
};

pub const ITEM_ON_HEAD: EntityState = EntityState {
    tag: "on_head",
    is_state_of: |e, state| {
        if let Some(slots) = EntityInsights::of(e, state).equipped_slots() {
            slots.contains(&EquipmentSlot::Head)
        } else {
            false
        }
    },
};

pub const ITEM_ON_TORSO: EntityState = EntityState {
    tag: "on_torso",
    is_state_of: |e, state| {
        if let Some(slots) = EntityInsights::of(e, state).equipped_slots() {
            slots.contains(&EquipmentSlot::Torso)
        } else {
            false
        }
    },
};

pub const ITEM_ON_LEGS: EntityState = EntityState {
    tag: "on_legs",
    is_state_of: |e, state| {
        if let Some(slots) = EntityInsights::of(e, state).equipped_slots() {
            slots.contains(&EquipmentSlot::Legs)
        } else {
            false
        }
    },
};

pub const ITEM_ON_HAND: EntityState = EntityState {
    tag: "on_hand",
    is_state_of: |e, state| {
        if let Some(slots) = EntityInsights::of(e, state).equipped_slots() {
            slots.contains(&EquipmentSlot::LeftHand) || slots.contains(&EquipmentSlot::RightHand)
        } else {
            false
        }
    },
};

pub const ITEM_ON_FEET: EntityState = EntityState {
    tag: "on_hand",
    is_state_of: |e, state| {
        if let Some(slots) = EntityInsights::of(e, state).equipped_slots() {
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
