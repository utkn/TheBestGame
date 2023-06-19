use std::collections::{HashMap, HashSet};

use crate::core::{
    EntityRef, EntityRefBag, EntityValiditySet, State, StateCommands, System, UpdateContext,
};

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
        occupied_slots: &HashSet<EquipmentSlot>,
    ) -> Option<HashSet<EquipmentSlot>> {
        let mut chosen_slots = HashSet::new();
        for clause in &self.0 {
            let chosen_slot = clause.iter().find(|option| {
                !occupied_slots.contains(*option) && !chosen_slots.contains(*option)
            })?;
            chosen_slots.insert(*chosen_slot);
        }
        Some(chosen_slots)
    }
}

/// An entity that can be equipped by `Equipment` entities.
#[derive(Clone, Debug)]
pub struct Equippable(pub SlotSelector);

/// An entity that can equip `Equippable` entities.
#[derive(Clone, Default, Debug)]
pub struct Equipment(HashMap<EquipmentSlot, EntityRef>);

impl Equipment {
    /// Returns the set of occupied slots in this equipment.
    pub fn occupied_slots(&self) -> HashSet<EquipmentSlot> {
        self.0.keys().cloned().collect()
    }
    /// Returns true if the item can be equipped.
    pub fn can_equip(&self, equippable: &Equippable) -> bool {
        equippable.0.choose_slots(&self.occupied_slots()).is_some()
    }

    /// Tries to equip the given entity to the given slot. Note that this always succeeds if the slots are available.
    pub fn try_equip(&mut self, e: EntityRef, equippable: &Equippable) -> bool {
        if let Some(chosen_slots) = equippable.0.choose_slots(&self.occupied_slots()) {
            self.0.extend(chosen_slots.iter().map(|slot| (*slot, e)));
            true
        } else {
            false
        }
    }

    pub fn get(&self, slot: EquipmentSlot) -> Option<&EntityRef> {
        self.0.get(&slot)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EquipmentSlot, &EntityRef)> {
        self.0.iter()
    }
}

impl EntityRefBag for Equipment {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef> {
        self.0
            .iter()
            .filter_map(|(_, v)| if !valids.is_valid(v) { Some(v) } else { None })
            .cloned()
            .collect()
    }

    fn try_remove_all(&mut self, entities: HashSet<EntityRef>) -> HashSet<EntityRef> {
        let entries_to_remove: HashSet<_> = self
            .0
            .iter()
            .filter_map(|(k, v)| {
                if entities.contains(v) {
                    Some((*k, *v))
                } else {
                    None
                }
            })
            .collect();
        entries_to_remove.iter().for_each(|(k, _)| {
            self.0.remove(k);
        });
        entries_to_remove.into_iter().map(|(_, v)| v).collect()
    }

    fn contains(&self, e: &EntityRef) -> bool {
        self.0.values().find(|v| *v == e).is_some()
    }

    fn try_remove(&mut self, e: &EntityRef) -> bool {
        let old_size = self.0.len();
        self.0.retain(|_, v| v != e);
        old_size != self.0.len()
    }
}

/// Request to equip an entity.
#[derive(Clone, Copy, Debug)]
pub struct EquipEntityReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Request to unequip an entity.
#[derive(Clone, Copy, Debug)]
pub struct UnequipEntityReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Emitted when an item is equipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityEquippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// Emitted when an item is unequipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityUnequippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

/// A system that handles equipping/unequpping to/from `Equipment` entities.
#[derive(Clone, Debug)]
pub struct EquipmentSystem;

impl System for EquipmentSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Keep a set of events to emit at the end of the execution.
        let mut unequipped_events = HashSet::<EntityUnequippedEvt>::new();
        let mut equipped_events = HashSet::<EntityEquippedEvt>::new();
        // Remove the invalid entities from equipment.
        let valids = state.extract_validity_set();
        state
            .select::<(Equipment,)>()
            .for_each(|(e, (equipment,))| {
                // Invalidated entities should be unequipped.
                let invalid_entities = equipment.get_invalids(&valids).into_iter();
                unequipped_events.extend(invalid_entities.map(|invalid_e| EntityUnequippedEvt {
                    entity: invalid_e,
                    equipment_entity: e,
                }));
            });
        // Handle the explicit requests.
        state.read_events::<UnequipEntityReq>().for_each(|evt| {
            // Decide whether we need to emit an event.
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.equipment_entity) {
                if equipment.contains(&evt.entity) {
                    unequipped_events.insert(EntityUnequippedEvt {
                        entity: evt.entity,
                        equipment_entity: evt.equipment_entity,
                    });
                }
            }
        });
        state.read_events::<EquipEntityReq>().for_each(|evt| {
            if let Some((equippable,)) = state.select_one::<(Equippable,)>(&evt.entity) {
                if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.equipment_entity)
                {
                    if equipment.can_equip(equippable) && evt.entity != evt.equipment_entity {
                        equipped_events.insert(EntityEquippedEvt {
                            entity: evt.entity,
                            equipment_entity: evt.equipment_entity,
                        });
                    }
                }
            }
        });
        // Emit the events.
        unequipped_events.into_iter().for_each(|evt| {
            cmds.emit_event(evt);
            cmds.update_component(&evt.equipment_entity, move |equipment: &mut Equipment| {
                equipment.try_remove(&evt.entity);
            });
        });
        equipped_events.into_iter().for_each(|evt| {
            if let Some((equippable,)) = state.select_one::<(Equippable,)>(&evt.entity) {
                let equippable = equippable.clone();
                cmds.emit_event(evt);
                cmds.update_component(&evt.equipment_entity, move |equipment: &mut Equipment| {
                    equipment.try_equip(evt.entity, &equippable);
                });
            }
        });
    }
}
