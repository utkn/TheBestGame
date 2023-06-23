use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::prelude::*;

use super::{ItemDescription, ItemInsights, ItemLocation, ItemStack, Storage};

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
        state: &State,
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
            .filter(|(_, item_stack)| item_stack.contains(item_entity))
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
}

impl EntityRefBag for Equipment {
    fn len(&self) -> usize {
        self.slots
            .values()
            .flat_map(|item_stack| item_stack.iter())
            .unique()
            .count()
    }

    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef> {
        self.slots
            .iter()
            .flat_map(|(_, item_stack)| item_stack.get_invalids(valids))
            .collect()
    }

    fn try_remove_all(&mut self, entities: &HashSet<EntityRef>) -> HashSet<EntityRef> {
        self.slots
            .values_mut()
            .flat_map(|item_stack| item_stack.try_remove_all(entities))
            .collect()
    }

    fn contains(&self, e: &EntityRef) -> bool {
        self.slots.values().any(|item_stack| item_stack.contains(e))
    }

    fn try_remove(&mut self, e: &EntityRef) -> bool {
        let old_size = self.len();
        self.slots.values_mut().for_each(|item_stack| {
            item_stack.try_remove(e);
        });
        old_size != self.len()
    }
}

/// An [`Equipment`] can act as an activation/unactivation [`Interaction`].
impl Interaction for Equipment {
    fn priority() -> usize {
        Storage::priority()
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        state.select_one::<(Equipment,)>(target).is_some()
            && state.select_one::<(Character,)>(actor).is_some()
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
            && StateInsights::of(state).location_of(target) == ItemLocation::Ground
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}

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
                    .try_remove(item_entity)
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
            let validity_set = state.extract_validity_set();
            cmds.update_component(&e, move |equipment: &mut Equipment| {
                equipment.remove_invalids(&validity_set);
            });
        })
    }
}
