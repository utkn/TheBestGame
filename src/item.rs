use crate::{
    core::*,
    entity_insights::{EntityInsights, EntityLocation},
    equipment::{EquipEntityReq, Equipment, Equippable, UnequipEntityReq},
    interaction::{
        InteractionStartedEvt, InteractionType, ProximityInteractable, TryUninteractTargetedReq,
    },
    storage::{Storage, StoreEntityReq, UnstoreEntityReq},
};

#[derive(Clone, Copy, Debug)]
pub struct Item;

#[derive(Clone, Copy, Debug)]
pub struct ItemTransferReq {
    pub item_entity: EntityRef,
    pub from_loc: EntityLocation,
    pub to_loc: EntityLocation,
}

impl ItemTransferReq {
    /// Constructs a pick up transfer request.
    pub fn pick_up(item_entity: EntityRef, target_storage: EntityRef) -> Self {
        Self {
            item_entity,
            from_loc: EntityLocation::Ground,
            to_loc: EntityLocation::Storage(target_storage),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemTransferEvt {
    pub item_entity: EntityRef,
    pub from_loc: EntityLocation,
    pub to_loc: EntityLocation,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemTransferSystem;

impl ItemTransferSystem {
    fn from_loc_valid(&self, item_e: &EntityRef, loc: &EntityLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        let curr_location = EntityLocation::of(item_e, state);
        is_item_entity_valid && curr_location == *loc
    }

    fn to_loc_valid(&self, item_e: &EntityRef, loc: &EntityLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        is_item_entity_valid
            && match loc {
                EntityLocation::Ground => true,
                EntityLocation::Equipment(equipment_entity) if equipment_entity != item_e => {
                    if let Some((equippable,)) = state.select_one::<(Equippable,)>(item_e) {
                        state
                            .select_one::<(Equipment,)>(equipment_entity)
                            .map(|(equipment,)| equipment.can_equip(&equippable))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                EntityLocation::Storage(storage_entity) if storage_entity != item_e => {
                    if let Some((_storage,)) = state.select_one::<(Storage,)>(storage_entity) {
                        // TODO: storage limits
                        true
                    } else {
                        false
                    }
                }
                _ => false,
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
                EntityLocation::Ground => {}
                EntityLocation::Equipment(equipment_entity) => cmds.emit_event(UnequipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                EntityLocation::Storage(storage_entity) => cmds.emit_event(UnstoreEntityReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            };
            // Place in the new location.
            match evt.to_loc {
                EntityLocation::Ground => {}
                EntityLocation::Equipment(equipment_entity) => cmds.emit_event(EquipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                EntityLocation::Storage(storage_entity) => cmds.emit_event(StoreEntityReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            }
            cmds.emit_event(ItemTransferEvt {
                from_loc: evt.from_loc,
                to_loc: evt.to_loc,
                item_entity: evt.item_entity,
            });
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemAnchorSystem;

impl System for ItemAnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<ItemTransferEvt>()
            .filter(|evt| evt.from_loc == EntityLocation::Ground)
            .filter_map(|evt| match evt.to_loc {
                EntityLocation::Ground => None,
                EntityLocation::Equipment(e) | EntityLocation::Storage(e) => {
                    Some((evt.item_entity, e))
                }
            })
            .filter(|(item, _)| state.select_one::<(Item,)>(item).is_some())
            .for_each(|(item, actor)| {
                cmds.set_components(
                    &item,
                    (Transform::default(), AnchorTransform(actor, (0., 0.))),
                );
            });
        state
            .read_events::<ItemTransferEvt>()
            .filter(|evt| evt.to_loc == EntityLocation::Ground)
            .filter_map(|evt| match evt.from_loc {
                EntityLocation::Ground => None,
                EntityLocation::Equipment(e) | EntityLocation::Storage(e) => {
                    Some((evt.item_entity, e))
                }
            })
            .filter(|(item, _)| state.select_one::<(Item,)>(item).is_some())
            .for_each(|(item, _)| {
                cmds.remove_component::<AnchorTransform>(&item);
            });
    }
}

impl InteractionType for Item {
    fn priority() -> usize {
        100
    }

    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let target_insights = EntityInsights::of(target, state);
        if target_insights.location != EntityLocation::Ground {
            return false;
        }
        if !target_insights.contacts.contains(actor) {
            return false;
        }
        if state.select_one::<(Item,)>(target).is_none() {
            return false;
        }
        if let Some((actor_storage,)) = state.select_one::<(Storage,)>(actor) {
            actor_storage.can_store(target, state)
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemPickupSystem;

impl System for ItemPickupSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<ItemTransferEvt>()
            // Going to ground
            .filter(|evt| evt.to_loc == EntityLocation::Ground)
            // From either equipment or storage
            .filter(|evt| {
                matches!(
                    evt.from_loc,
                    EntityLocation::Equipment(_) | EntityLocation::Storage(_)
                )
            })
            .map(|evt| evt.item_entity)
            .filter(|item| state.select_one::<(Item,)>(item).is_some())
            .for_each(|item| {
                cmds.set_component(&item, ProximityInteractable);
            });
        state
            .read_events::<InteractionStartedEvt<Item>>()
            .for_each(|evt| {
                cmds.emit_event(ItemTransferReq::pick_up(evt.target, evt.actor));
                cmds.remove_component::<ProximityInteractable>(&evt.target);
                // Stop the interaction immediately.
                cmds.emit_event(TryUninteractTargetedReq::<Item>::new(evt.actor, evt.target));
            });
    }
}
