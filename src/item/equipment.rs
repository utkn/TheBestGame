use std::collections::{HashMap, HashSet};

use crate::prelude::*;

use super::{ItemDescription, ItemStack};

mod equipment_activation;
mod equipment_insights;
mod equipment_slot;
mod equipment_system;

pub use equipment_activation::*;
pub use equipment_insights::*;
pub use equipment_slot::*;
pub use equipment_system::*;

/// An entity that can be equipped by [`Equipment`] entities.
#[derive(Clone, Debug)]
pub struct Equippable(pub SlotSelector);

/// An entity that can equip [`Equippable`] entities.
#[derive(Clone, Debug)]
pub struct Equipment {
    slots: HashMap<EquipmentSlot, ItemStack>,
}

impl Equipment {
    pub fn new(accepting_slots: impl IntoIterator<Item = EquipmentSlot>) -> Self {
        let slots = HashMap::from_iter(
            accepting_slots
                .into_iter()
                .map(|slot| (slot, ItemStack::one())),
        );
        Self { slots }
    }

    pub fn slots(&self) -> impl Iterator<Item = (&EquipmentSlot, &ItemStack)> {
        self.slots
            .iter()
            .map(|(eq_slot, item_stack)| (eq_slot, item_stack))
    }

    pub fn content_description<'a>(
        &'a self,
        state: &'a State,
    ) -> HashMap<EquipmentSlot, ItemDescription<'a>> {
        self.slots
            .iter()
            .filter_map(|(eq_slot, item_stack)| {
                if let Some(desc) = item_stack.head_item_description(state) {
                    Some((*eq_slot, desc))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the set of equipment slots that this `item_entity` will occupy. Returns `None` if it cannot be equipped.
    pub fn get_slots_to_occupy(
        &self,
        item_entity: &EntityRef,
        state: &State,
    ) -> Option<HashSet<EquipmentSlot>> {
        let slot_selector = &state.select_one::<(Equippable,)>(item_entity)?.0 .0;
        slot_selector.choose_slots(item_entity, &self.slots, state)
    }

    /// Returns the set of equipment slots that the given `item_entity` is stored in.
    pub fn get_containing_slots(&self, item_entity: &EntityRef) -> Option<HashSet<EquipmentSlot>> {
        let occupied_slots: HashSet<_> = self
            .slots
            .iter()
            .filter(|(_, item_stack)| item_stack.items().contains(item_entity))
            .map(|(equipment_slot, _)| *equipment_slot)
            .collect();
        if occupied_slots.len() == 0 {
            None
        } else {
            Some(occupied_slots)
        }
    }

    /// Returns the [`ItemStack`] at the given `equipment_slot`.
    pub fn get_item_stack(&self, equipment_slot: &EquipmentSlot) -> Option<&ItemStack> {
        self.slots.get(equipment_slot)
    }

    pub fn contains(&self, e: &EntityRef) -> bool {
        self.slots
            .values()
            .any(|item_stack| item_stack.items().contains(e))
    }
}

impl EntityRefBag for Equipment {
    fn remove_invalids(&mut self, entity_mgr: &EntityManager) {
        self.slots.values_mut().for_each(|item_stack| {
            item_stack.remove_invalids(entity_mgr);
        });
    }
}
