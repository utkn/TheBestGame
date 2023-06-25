use crate::{
    item::ItemStack,
    prelude::{EntityRef, StateInsights},
};

use super::{Equipment, EquipmentSlot};

pub trait EquipmentInsights<'a> {
    /// Returns true iff the given `item_entity` can be equipped by `equipment_entity`.
    fn can_equip(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being equipped by this entity.
    fn is_equipping(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given entity has an equipment.
    fn has_equipment(&self, e: &EntityRef) -> bool;
    /// Returns the equippable item at the given slot.
    fn equippable_at(
        &self,
        equipment_entity: &EntityRef,
        slot: &EquipmentSlot,
    ) -> Option<&'a ItemStack>;
}

impl<'a> EquipmentInsights<'a> for StateInsights<'a> {
    fn has_equipment(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Equipment,)>(e).is_some()
    }

    fn equippable_at(
        &self,
        equipment_entity: &EntityRef,
        slot: &EquipmentSlot,
    ) -> Option<&'a ItemStack> {
        self.0
            .select_one::<(Equipment,)>(equipment_entity)?
            .0
            .get_item_stack(slot)
    }

    fn can_equip(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Equipment,)>(equipment_entity)
            .map(|(equipment,)| equipment.get_slots_to_occupy(item_entity, self.0).is_some())
            .unwrap_or(false)
    }

    fn is_equipping(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Equipment,)>(equipment_entity)
            .map(|(equipment,)| equipment.contains(item_entity))
            .unwrap_or(false)
    }
}
