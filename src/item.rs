use std::collections::HashSet;

use crate::{
    core::*,
    entity_insights::{EntityInsights, EntityLocation},
    equipment::{
        EntityEquippedEvt, EntityUnequippedEvt, EquipEntityReq, Equipment, Equippable,
        UnequipEntityReq,
    },
    interaction::InteractionType,
    physics::CollisionState,
    storage::{Storage, StoreEntityReq, UnstoreEntityReq},
};

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

    pub fn drop(item_entity: EntityRef, from_storage: EntityRef) -> Self {
        Self {
            item_entity,
            to_loc: EntityLocation::Ground,
            from_loc: EntityLocation::Storage(from_storage),
        }
    }

    pub fn unequip(
        item_entity: EntityRef,
        from_equipment: EntityRef,
        to_storage: EntityRef,
    ) -> Self {
        Self {
            item_entity,
            to_loc: EntityLocation::Storage(to_storage),
            from_loc: EntityLocation::Equipment(from_equipment),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Item;

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
                EntityLocation::Ground => {
                    cmds.remove_component::<Transform>(&evt.item_entity);
                }
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
                EntityLocation::Ground => {
                    let new_transform = match evt.from_loc {
                        EntityLocation::Ground => (Transform::default(),),
                        EntityLocation::Equipment(entity) | EntityLocation::Storage(entity) => {
                            state
                                .select_one::<(Transform,)>(&entity)
                                .map(|(trans,)| (*trans,))
                                .unwrap_or_default()
                        }
                    };
                    cmds.set_components(&evt.item_entity, new_transform);
                }
                EntityLocation::Equipment(equipment_entity) => cmds.emit_event(EquipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                EntityLocation::Storage(storage_entity) => cmds.emit_event(StoreEntityReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EquippedItemAnchorSystem;

impl System for EquippedItemAnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<EntityEquippedEvt>().for_each(|evt| {
            if let (Some(_), Some(_)) = (
                state.select_one::<(Equipment,)>(&evt.equipment_entity),
                state.select_one::<(Item,)>(&evt.entity),
            ) {
                cmds.set_components(
                    &evt.entity,
                    (
                        Transform::default(),
                        AnchorTransform(evt.equipment_entity, (0., 0.)),
                    ),
                );
            }
        });
        state.read_events::<EntityUnequippedEvt>().for_each(|evt| {
            if let (Some(_), Some(_)) = (
                state.select_one::<(Equipment,)>(&evt.equipment_entity),
                state.select_one::<(Item,)>(&evt.entity),
            ) {
                cmds.remove_component::<AnchorTransform>(&evt.entity);
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemPickupInteraction;

impl InteractionType for ItemPickupInteraction {
    fn valid_actors(target: &EntityRef, state: &State) -> Option<HashSet<EntityRef>> {
        if EntityInsights::of(target, state).location != EntityLocation::Ground {
            return None;
        }
        let (_, _, target_coll_state) =
            state.select_one::<(Transform, Item, CollisionState)>(target)?;
        let picker_actors: HashSet<_> = target_coll_state
            .colliding
            .iter()
            .filter_map(|actor| {
                state
                    .select_one::<(Storage,)>(actor)
                    .map(|(actor_storage,)| (actor, actor_storage))
            })
            .filter(|(_, actor_storage)| actor_storage.can_store(target, state))
            .map(|(actor, _)| *actor)
            .collect();
        if picker_actors.len() > 0 {
            Some(picker_actors)
        } else {
            None
        }
    }

    fn should_end(_: &EntityRef, _: &EntityRef, _: &State) -> bool {
        // one shot
        true
    }

    fn on_start(actor: &EntityRef, target: &EntityRef, _state: &State, cmds: &mut StateCommands) {
        cmds.emit_event(ItemTransferReq::pick_up(*target, *actor));
    }

    fn priority() -> usize {
        100
    }
}
