use std::collections::HashSet;

use crate::prelude::{EntityRef, EntityRefBag, StateInsights};

use super::{
    Equipment, EquipmentSlot, Item, ItemEquippedEvt, ItemLocation, ItemStoredEvt,
    ItemUnequippedEvt, ItemUnstoredEvt, Storage,
};

/// Provides insights about an entity that could be possibly be an item.
pub trait ItemInsights {
    /// Returns true if the given entity is indeed an item.
    fn is_item(&self, e: &EntityRef) -> bool;
    /// Returns the location of the item.
    fn location_of(&self, item_entity: &EntityRef) -> ItemLocation;
    /// Returns the set of entities that stored this item in the last update.
    fn new_storers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef>;
    /// Returns the set of entities that unstored this item in the last update.
    fn new_unstorers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef>;
    /// Returns the set of entities that equipped this item in the last update.
    fn new_equippers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef>;
    /// Returns the set of entities that unequipped this item in the last update.
    fn new_unequippers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef>;
    /// Returns the storer of this item entity.
    fn storer_of(&self, item_entity: &EntityRef) -> Option<EntityRef>;
    /// Returns the equipper of this item entity.
    fn equipper_of(&self, item_entity: &EntityRef) -> Option<EntityRef>;
    /// Returns the slots that this item entity is equipped in.
    fn equipped_slots_of(&self, item_entity: &EntityRef) -> Option<HashSet<EquipmentSlot>>;
}

impl<'a> ItemInsights for StateInsights<'a> {
    fn is_item(&self, item_entity: &EntityRef) -> bool {
        self.0.select_one::<(Item,)>(item_entity).is_some()
    }

    fn location_of(&self, item_entity: &EntityRef) -> ItemLocation {
        if let Some((storing_entity, _)) = self
            .0
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.contains(item_entity))
        {
            ItemLocation::Storage(storing_entity)
        } else if let Some((equipping_entity, _)) = self
            .0
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(item_entity))
        {
            ItemLocation::Equipment(equipping_entity)
        } else {
            ItemLocation::Ground
        }
    }

    fn new_storers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef> {
        self.0
            .read_events::<ItemStoredEvt>()
            .filter(|evt| &evt.entity == item_entity)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_unstorers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef> {
        self.0
            .read_events::<ItemUnstoredEvt>()
            .filter(|evt| &evt.entity == item_entity)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_equippers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef> {
        self.0
            .read_events::<ItemEquippedEvt>()
            .filter(|evt| &evt.entity == item_entity)
            .map(|evt| evt.equipment_entity)
            .collect()
    }

    fn new_unequippers_of(&self, item_entity: &EntityRef) -> HashSet<EntityRef> {
        self.0
            .read_events::<ItemUnequippedEvt>()
            .filter(|evt| &evt.entity == item_entity)
            .map(|evt| evt.equipment_entity)
            .collect()
    }

    fn storer_of(&self, item_entity: &EntityRef) -> Option<EntityRef> {
        self.0
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.contains(item_entity))
            .map(|(e, _)| e)
    }

    fn equipper_of(&self, item_entity: &EntityRef) -> Option<EntityRef> {
        self.0
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(item_entity))
            .map(|(e, _)| e)
    }

    fn equipped_slots_of(&self, item_entity: &EntityRef) -> Option<HashSet<EquipmentSlot>> {
        self.0
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(item_entity))
            .and_then(|(_, (equipment,))| equipment.get_containing_slots(item_entity))
    }
}

pub trait StorageInsights {
    /// Returns true iff the given `item_entity` can be stored by `storage_entity`.
    fn can_store(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being stored by `storage_entity`.
    fn is_storing(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool;
}

impl<'a> StorageInsights for StateInsights<'a> {
    fn can_store(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Storage,)>(storage_entity)
            .map(|(storage,)| storage.get_available_slot(item_entity, self.0).is_some())
            .unwrap_or(false)
    }

    fn is_storing(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Storage,)>(storage_entity)
            .map(|(storage,)| storage.contains(item_entity))
            .unwrap_or(false)
    }
}

pub trait EquipmentInsights {
    /// Returns true iff the given `item_entity` can be equipped by `equipment_entity`.
    fn can_equip(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being equipped by this entity.
    fn is_equipping(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool;
}

impl<'a> EquipmentInsights for StateInsights<'a> {
    fn can_equip(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Equipment,)>(equipment_entity)
            .map(|(equipment,)| equipment.get_slots_to_occupy(item_entity, self.0).is_some())
            .unwrap_or(false)
    }

    fn is_equipping(&self, equipment_entity: &EntityRef, item_entity: &EntityRef) -> bool {
        self.0
            .select_one::<(Equipment,)>(equipment_entity)
            .map(|(equipment,)| equipment.contains(item_entity))
            .unwrap_or(false)
    }
}
