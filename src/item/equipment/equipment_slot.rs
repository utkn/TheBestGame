use std::collections::{HashMap, HashSet};

use crate::{item::ItemStack, prelude::*};

/// Represent a slot in the equipment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Head,
    Torso,
    Legs,
    Feet,
    Accessory(u8),
    Backpack,
    LeftHand,
    RightHand,
    WeaponAmmo,
    WeaponModule,
    VehicleGas,
    VehicleModule,
}

impl From<EquipmentSlot> for &'static str {
    fn from(slot: EquipmentSlot) -> Self {
        match slot {
            EquipmentSlot::Head => "head",
            EquipmentSlot::Torso => "torso",
            EquipmentSlot::Legs => "legs",
            EquipmentSlot::Feet => "feet",
            EquipmentSlot::Accessory(_) => "accessory",
            EquipmentSlot::Backpack => "backpack",
            EquipmentSlot::LeftHand => "left_hand",
            EquipmentSlot::RightHand => "right_hand",
            EquipmentSlot::WeaponAmmo => "weapon_ammo",
            EquipmentSlot::WeaponModule => "weapon_module",
            EquipmentSlot::VehicleGas => "vehicle_gas",
            EquipmentSlot::VehicleModule => "vehicle_module",
        }
    }
}

/// Denotes the slots that an item can occupy.
#[derive(Clone, Debug)]
pub struct SlotSelector(Vec<Vec<EquipmentSlot>>);

impl SlotSelector {
    pub fn new<C, E>(clauses: C) -> Self
    where
        C: IntoIterator<Item = E>,
        E: IntoIterator<Item = EquipmentSlot>,
    {
        let clauses = clauses
            .into_iter()
            .map(|clause| clause.into_iter().collect())
            .collect();
        Self(clauses)
    }

    /// Chooses a set of slots from the given occupied slots. Returns `None` if the selection fails.
    pub fn choose_slots<'a>(
        &self,
        item_entity: &EntityRef,
        slots: &HashMap<EquipmentSlot, ItemStack>,
        state: &impl StateReader,
    ) -> Option<HashSet<EquipmentSlot>> {
        let mut chosen_slots = HashSet::new();
        for clause in &self.0 {
            let chosen_slot = clause.iter().find(|option| {
                slots.contains_key(*option)
                    && slots
                        .get(*option)
                        .map(|item_stack| item_stack.can_store(item_entity, state))
                        .unwrap_or(true)
                    && !chosen_slots.contains(*option)
            })?;
            chosen_slots.insert(*chosen_slot);
        }
        Some(chosen_slots)
    }
}
