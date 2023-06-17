use std::collections::{HashMap, HashSet};

use crate::core::{
    EntityRef, EntityRefStorage, EntityValiditySet, State, StateCommands, System, UpdateContext,
};

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

#[derive(Clone, Debug)]
pub struct Equippable(pub HashSet<EquipmentSlot>);

impl Equippable {
    pub fn new(slots: impl IntoIterator<Item = EquipmentSlot>) -> Self {
        Self(HashSet::from_iter(slots.into_iter()))
    }
}

#[derive(Clone, Default, Debug)]
pub struct Equipment(HashMap<EquipmentSlot, EntityRef>);

impl Equipment {
    /// Returns true if the item can be equipped.
    pub fn can_equip(&self, equippable: &Equippable) -> bool {
        equippable.0.iter().all(|slot| !self.0.contains_key(slot))
    }

    /// Tries to equip the given entity to the given slot. Note that this always succeeds if the slots are available.
    pub fn try_equip(&mut self, e: EntityRef, equippable: &Equippable) -> bool {
        if self.can_equip(equippable) {
            self.0.extend(equippable.0.iter().map(|slot| (*slot, e)));
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

impl EntityRefStorage for Equipment {
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

#[derive(Clone, Copy, Debug)]
pub struct EquipEntityReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct UnequipEntityReq {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityEquippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityUnequippedEvt {
    pub entity: EntityRef,
    pub equipment_entity: EntityRef,
}

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
                    if equipment.can_equip(equippable) {
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
