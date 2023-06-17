use crate::{
    core::*,
    equipment::{EquipEntityReq, Equipment, Equippable, UnequipEntityReq},
    interaction::InteractionStartedEvt,
    storage::{Storage, StoreEntityReq, UnstoreEntityReq},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemLocation {
    Ground,
    Equipment(EntityRef),
    Storage(EntityRef),
}

#[derive(Clone, Copy, Debug)]
pub struct ItemTransferReq {
    pub item_entity: EntityRef,
    pub from_loc: ItemLocation,
    pub to_loc: ItemLocation,
}

impl ItemTransferReq {
    /// Constructs a pick up transfer request.
    pub fn pick_up(item_entity: EntityRef, target_storage: EntityRef) -> Self {
        Self {
            item_entity,
            from_loc: ItemLocation::Ground,
            to_loc: ItemLocation::Storage(target_storage),
        }
    }

    pub fn drop(item_entity: EntityRef, from_storage: EntityRef) -> Self {
        Self {
            item_entity,
            to_loc: ItemLocation::Ground,
            from_loc: ItemLocation::Storage(from_storage),
        }
    }

    pub fn unequip(
        item_entity: EntityRef,
        from_equipment: EntityRef,
        to_storage: EntityRef,
    ) -> Self {
        Self {
            item_entity,
            to_loc: ItemLocation::Storage(to_storage),
            from_loc: ItemLocation::Equipment(from_equipment),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Item;

#[derive(Clone, Copy, Debug)]
pub struct ItemTransferSystem;

impl ItemTransferSystem {
    fn from_loc_valid(&self, item_e: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        is_item_entity_valid
            && match loc {
                ItemLocation::Ground => {
                    let in_no_storage = state
                        .select::<(Storage,)>()
                        .all(|(_, (storage,))| !storage.0.contains(item_e));
                    let in_no_equipment = state
                        .select::<(Equipment,)>()
                        .all(|(_, (equipment,))| !equipment.contains(item_e));
                    in_no_storage && in_no_equipment
                }
                ItemLocation::Equipment(equipment_entity) => {
                    state.select_one::<(Equippable,)>(item_e).is_some()
                        && state
                            .select_one::<(Equipment,)>(equipment_entity)
                            .map(|(equipment,)| equipment.contains(item_e))
                            .unwrap_or(false)
                }
                ItemLocation::Storage(storage_entity) => state
                    .select_one::<(Storage,)>(storage_entity)
                    .map(|(storage,)| storage.0.contains(item_e))
                    .unwrap_or(false),
            }
    }

    fn to_loc_valid(&self, item_e: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        is_item_entity_valid
            && match loc {
                ItemLocation::Ground => true,
                ItemLocation::Equipment(equipment_entity) => {
                    if let Some((equippable,)) = state.select_one::<(Equippable,)>(item_e) {
                        state
                            .select_one::<(Equipment,)>(equipment_entity)
                            .map(|(equipment,)| equipment.can_equip(&equippable))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                ItemLocation::Storage(storage_entity) => {
                    if let Some((_storage,)) = state.select_one::<(Storage,)>(storage_entity) {
                        // TODO: storage limits
                        true
                    } else {
                        false
                    }
                }
            }
    }
}

impl System for ItemTransferSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<ItemTransferReq>().for_each(|evt| {
            let is_valid_from_loc = self.from_loc_valid(&evt.item_entity, &evt.from_loc, state);
            let is_valid_to_loc = self.to_loc_valid(&evt.item_entity, &evt.to_loc, state);
            if !is_valid_from_loc || !is_valid_to_loc || evt.from_loc == evt.to_loc {
                return;
            }
            // Remove from the current location.
            match evt.from_loc {
                ItemLocation::Ground => {
                    cmds.remove_component::<Position>(&evt.item_entity);
                }
                ItemLocation::Equipment(equipment_entity) => cmds.emit_event(UnequipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                ItemLocation::Storage(storage_entity) => cmds.emit_event(UnstoreEntityReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            };
            // Place in the new location.
            match evt.to_loc {
                ItemLocation::Ground => {
                    let new_position = match evt.from_loc {
                        ItemLocation::Ground => Position::default(),
                        ItemLocation::Equipment(pos_entity) | ItemLocation::Storage(pos_entity) => {
                            state
                                .select_one::<(Position,)>(&pos_entity)
                                .map(|(pos,)| *pos)
                                .unwrap_or_default()
                        }
                    };
                    cmds.set_component(&evt.item_entity, new_position);
                }
                ItemLocation::Equipment(equipment_entity) => cmds.emit_event(EquipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                ItemLocation::Storage(storage_entity) => cmds.emit_event(StoreEntityReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemPickupSystem;

impl System for ItemPickupSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<InteractionStartedEvt>()
            .for_each(|evt| {
                if let (Some(_), Some(_)) = (
                    state.select_one::<(Storage,)>(&evt.0.actor),
                    state.select_one::<(Position, Item)>(&evt.0.target),
                ) {
                    cmds.emit_event(ItemTransferReq::pick_up(evt.0.target, evt.0.actor));
                }
            });
    }
}
