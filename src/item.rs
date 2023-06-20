use crate::{
    interaction::{
        Interaction, InteractionStartedEvt, ProximityInteractable, TryUninteractTargetedReq,
    },
    physics::ColliderInsights,
    prelude::*,
};

pub use equipment::*;
pub use item_insights::*;
pub use storage::*;

mod equipment;
mod item_insights;
mod storage;

/// Represents an entity that can be equipped, stored, and dropped on the ground.
#[derive(Clone, Copy, Debug)]
pub struct Item;

/// Represents the location of an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemLocation {
    Ground,
    Equipment(EntityRef),
    Storage(EntityRef),
}

/// A request to transfer an item entity between locations. Handled by [`ItemTransferSystem`].
#[derive(Clone, Copy, Debug)]
pub struct ItemTransferReq {
    /// The item entity to transfer.
    pub item_entity: EntityRef,
    /// Current location.
    pub from_loc: ItemLocation,
    /// Requested location.
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
}

/// Emitted by [`ItemTransferSystem`] when an item transfer occurs.
#[derive(Clone, Copy, Debug)]
pub struct ItemTransferEvt {
    pub item_entity: EntityRef,
    pub from_loc: ItemLocation,
    pub to_loc: ItemLocation,
}

/// A system that handles item transfers by listening to [`ItemTransferReq`]s and emitting [`ItemTransferEvt`]s.
#[derive(Clone, Copy, Debug)]
pub struct ItemTransferSystem;

impl ItemTransferSystem {
    /// Returns true if the given item is indeed in the given location.
    fn from_loc_valid(&self, item_e: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        let curr_location = EntityInsights::of(item_e, state).location();
        is_item_entity_valid && curr_location == *loc
    }

    /// Returns true if the given item can be moved to the given location.
    fn to_loc_valid(&self, item_e: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_e).is_some();
        is_item_entity_valid
            && match loc {
                ItemLocation::Ground => true,
                // Check if the item is equippable by the target [`Equipment`].
                ItemLocation::Equipment(equipment_entity) if equipment_entity != item_e => {
                    if let Some((equippable,)) = state.select_one::<(Equippable,)>(item_e) {
                        state
                            .select_one::<(Equipment,)>(equipment_entity)
                            .map(|(equipment,)| equipment.can_equip(&equippable))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                // Check if the item is storable by the target [`Storage`].
                ItemLocation::Storage(storage_entity) if storage_entity != item_e => {
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
                ItemLocation::Ground => {}
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
                ItemLocation::Ground => {}
                ItemLocation::Equipment(equipment_entity) => cmds.emit_event(EquipEntityReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                ItemLocation::Storage(storage_entity) => cmds.emit_event(StoreEntityReq {
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

/// A system that anchors the item's transformation to the [`Storage`] or [`Equipment`] that it is in.
#[derive(Clone, Copy, Debug)]
pub struct ItemAnchorSystem;

impl System for ItemAnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Handle transfer from equipment/storage.
        state
            .read_events::<ItemTransferEvt>()
            .filter(|evt| evt.to_loc == ItemLocation::Ground)
            .filter_map(|evt| match evt.from_loc {
                ItemLocation::Ground => None,
                ItemLocation::Equipment(e) | ItemLocation::Storage(e) => Some((evt.item_entity, e)),
            })
            .filter(|(item, _)| state.select_one::<(Item,)>(item).is_some())
            .for_each(|(item, _)| {
                cmds.remove_component::<AnchorTransform>(&item);
            });
        // Handle transfer to equipment/storage.
        state
            .read_events::<ItemTransferEvt>()
            .filter(|evt| evt.from_loc == ItemLocation::Ground)
            .filter_map(|evt| match evt.to_loc {
                ItemLocation::Ground => None,
                ItemLocation::Equipment(e) | ItemLocation::Storage(e) => Some((evt.item_entity, e)),
            })
            .filter(|(item, _)| state.select_one::<(Item,)>(item).is_some())
            .for_each(|(item, actor)| {
                cmds.set_components(
                    &item,
                    (Transform::default(), AnchorTransform(actor, (0., 0.))),
                );
            });
    }
}

/// [`Item`]s can be interacted with to pick them up.
impl Interaction for Item {
    fn priority() -> usize {
        100
    }

    fn can_start(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let target_insights = EntityInsights::of(target, state);
        if target_insights.location() != ItemLocation::Ground {
            return false;
        }
        if !target_insights.contacts().contains(actor) {
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

/// Handles [`Item`] interaction, i.e., item pick ups.
#[derive(Clone, Copy, Debug)]
pub struct ItemPickupSystem;

impl System for ItemPickupSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<ItemTransferEvt>()
            // Going to ground
            .filter(|evt| evt.to_loc == ItemLocation::Ground)
            // From either equipment or storage
            .filter(|evt| {
                matches!(
                    evt.from_loc,
                    ItemLocation::Equipment(_) | ItemLocation::Storage(_)
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
