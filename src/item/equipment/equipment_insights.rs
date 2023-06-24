use crate::{
    item::ItemStack,
    prelude::{EntityRef, StateInsights},
};

use super::{Equipment, EquipmentSlot};

pub trait EquipmentInsights<'a> {
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
}
