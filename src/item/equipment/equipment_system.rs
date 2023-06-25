use std::collections::HashMap;

use crate::{item::ItemStack, prelude::*};

use super::Equipment;

struct ShadowEquipment(Equipment);

impl From<Equipment> for ShadowEquipment {
    fn from(equipment: Equipment) -> Self {
        Self(equipment)
    }
}

impl ShadowEquipment {
    /// Tries to place the given entity in the underlying equipment and returns `true` iff it succeeds.
    fn try_equip(&mut self, item_entity: EntityRef, state: &State) -> bool {
        if let Some(eq_slots) = self.0.get_slots_to_occupy(&item_entity, state) {
            eq_slots.into_iter().for_each(|eq_slot| {
                self.0
                    .slots
                    .entry(eq_slot)
                    .or_insert(ItemStack::one())
                    .try_store(item_entity, state);
            });
            true
        } else {
            false
        }
    }

    /// Tries to remove the given entity from the underlying equipment and returns `true` iff it succeeds.
    fn try_unequip(&mut self, item_entity: &EntityRef) -> bool {
        if let Some(eq_slots) = self.0.get_containing_slots(item_entity) {
            eq_slots.into_iter().all(|eq_slot| {
                self.0
                    .slots
                    .entry(eq_slot)
                    .or_insert(ItemStack::one())
                    .items_mut()
                    .remove(item_entity)
            })
        } else {
            false
        }
    }

    fn take(self) -> Equipment {
        self.0
    }
}
/// Request to equip an entity.
#[derive(Clone, Copy, Debug)]
pub struct EquipItemReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Request to unequip an entity.
#[derive(Clone, Copy, Debug)]
pub struct UnequipItemReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Emitted when an item is equipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemEquippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Emitted when an item is unequipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemUnequippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// A system that handles equipping/unequpping to/from `Equipment` entities.
#[derive(Clone, Debug)]
pub struct EquipmentSystem;

impl System for EquipmentSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Maintain the shadow equipments.
        let mut shadow_equipment_map = HashMap::<EntityRef, ShadowEquipment>::new();
        // Perform the unequippings on the shadow equipments.
        state.read_events::<UnequipItemReq>().for_each(|evt| {
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.equipment_entity) {
                let shadow_eq = shadow_equipment_map
                    .entry(evt.equipment_entity)
                    .or_insert(ShadowEquipment::from(equipment.clone()));
                if shadow_eq.try_unequip(&evt.entity) {
                    cmds.emit_event(ItemUnequippedEvt {
                        equipment_entity: evt.equipment_entity,
                        entity: evt.entity,
                    })
                }
            }
        });
        // Perform the equippings on the shadow equipments.
        state.read_events::<EquipItemReq>().for_each(|evt| {
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.equipment_entity) {
                let shadow_eq = shadow_equipment_map
                    .entry(evt.equipment_entity)
                    .or_insert(ShadowEquipment::from(equipment.clone()));
                if shadow_eq.try_equip(evt.entity, state) {
                    cmds.emit_event(ItemEquippedEvt {
                        equipment_entity: evt.equipment_entity,
                        entity: evt.entity,
                    })
                }
            }
        });
        // Move the shadow equipments into the game.
        shadow_equipment_map
            .into_iter()
            .for_each(|(equipment_entity, shadow_equipment)| {
                cmds.set_component(&equipment_entity, shadow_equipment.take());
            });
        // Now, remove the invalids from all the equipments.
        state.select::<(Equipment,)>().for_each(|(e, _)| {
            cmds.remove_invalids::<Equipment>(&e);
        })
    }
}
