use crate::{character::CharacterInsights, controller::ProximityInteractable, prelude::*};

pub use create_item::*;
pub use equipment::*;
pub use item_description::*;
pub use item_insights::*;
pub use item_stack::*;
pub use item_tags::*;
pub use storage::*;

mod create_item;
mod equipment;
mod item_description;
mod item_insights;
mod item_stack;
mod item_tags;
mod storage;

/// Represents an entity that can be equipped, stored, and dropped on the ground.
#[derive(Clone, Copy, Debug)]
pub struct Item(f32);

impl Item {
    /// Creates a new unstackable item.
    pub fn unstackable() -> Self {
        Self(ITEM_STACK_MAX_WEIGHT)
    }

    /// Creates a stackable item with maximum stack size `stack_size`.
    pub fn stackable(stack_size: usize) -> Self {
        Self(ITEM_STACK_MAX_WEIGHT / (stack_size as f32))
    }
}

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

    /// Constructs an equip from ground request.
    pub fn equip_from_ground(item_entity: EntityRef, target_equipment: EntityRef) -> Self {
        Self {
            item_entity,
            from_loc: ItemLocation::Ground,
            to_loc: ItemLocation::Equipment(target_equipment),
        }
    }

    /// Constructs an equip from ground request for the `target`.
    pub fn equip_from_storage(item_entity: EntityRef, target: EntityRef) -> Self {
        Self {
            item_entity,
            from_loc: ItemLocation::Storage(target),
            to_loc: ItemLocation::Equipment(target),
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
    fn from_loc_valid(&self, item_entity: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_entity).is_some();
        let curr_location = StateInsights::of(state).location_of(item_entity);
        is_item_entity_valid && curr_location == *loc
    }

    /// Returns true if the given item can be moved to the given location.
    fn to_loc_valid(&self, item_entity: &EntityRef, loc: &ItemLocation, state: &State) -> bool {
        let is_item_entity_valid = state.select_one::<(Item,)>(item_entity).is_some();
        is_item_entity_valid
            && match loc {
                ItemLocation::Ground => true,
                // Check if the item is equippable by the target [`Equipment`].
                ItemLocation::Equipment(equipment_entity) if equipment_entity != item_entity => {
                    StateInsights::of(state).can_equip(equipment_entity, item_entity)
                }
                // Check if the item is storable by the target [`Storage`].
                ItemLocation::Storage(storage_entity) if storage_entity != item_entity => {
                    StateInsights::of(state).can_store(storage_entity, item_entity)
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
                ItemLocation::Equipment(equipment_entity) => cmds.emit_event(UnequipItemReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                ItemLocation::Storage(storage_entity) => cmds.emit_event(UnstoreItemReq {
                    entity: evt.item_entity,
                    storage_entity,
                }),
            };
            // Place in the new location.
            match evt.to_loc {
                ItemLocation::Ground => {}
                ItemLocation::Equipment(equipment_entity) => cmds.emit_event(EquipItemReq {
                    entity: evt.item_entity,
                    equipment_entity,
                }),
                ItemLocation::Storage(storage_entity) => cmds.emit_event(StoreItemReq {
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
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
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

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        let insights = StateInsights::of(state);
        insights.is_item(target)
            && insights.location_of(target) == ItemLocation::Ground
            && insights.is_character(actor)
            && insights.can_store(actor, target)
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
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
                cmds.emit_event(UninteractReq::<Item>::new(evt.actor, evt.target));
            });
    }
}
