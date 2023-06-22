use std::collections::HashSet;

use crate::prelude::{EntityInsights, EntityRef, EntityRefBag};

use super::{
    Equipment, EquipmentSlot, Item, ItemEquippedEvt, ItemLocation, ItemStoredEvt,
    ItemUnequippedEvt, ItemUnstoredEvt, Storage,
};

/// Provides insights about an entity that could be possibly be an item.
pub trait ItemInsights {
    /// Returns true if the entity is indeed an item.
    fn is_item(&self) -> bool;
    /// Returns the location of the item.
    fn location(&self) -> ItemLocation;
    /// Returns the set of entities that stored this item in the last update.
    fn new_storers(&self) -> HashSet<EntityRef>;
    /// Returns the set of entities that unstored this item in the last update.
    fn new_unstorers(&self) -> HashSet<EntityRef>;
    /// Returns the set of entities that equipped this item in the last update.
    fn new_equippers(&self) -> HashSet<EntityRef>;
    /// Returns the set of entities that unequipped this item in the last update.
    fn new_unequippers(&self) -> HashSet<EntityRef>;
    /// Returns the storer of this item entity.
    fn storer(&self) -> Option<EntityRef>;
    /// Returns the equipper of this item entity.
    fn equipper(&self) -> Option<EntityRef>;
    /// Returns the slots that this item entity is equipped in.
    fn equipped_slots(&self) -> Option<HashSet<EquipmentSlot>>;
}

impl<'a> ItemInsights for EntityInsights<'a> {
    fn is_item(&self) -> bool {
        self.1.select_one::<(Item,)>(self.0).is_some()
    }

    fn location(&self) -> ItemLocation {
        if let Some((storing_entity, _)) = self
            .1
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.contains(self.0))
        {
            ItemLocation::Storage(storing_entity)
        } else if let Some((equipping_entity, _)) = self
            .1
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(self.0))
        {
            ItemLocation::Equipment(equipping_entity)
        } else {
            ItemLocation::Ground
        }
    }

    fn new_storers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<ItemStoredEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_unstorers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<ItemUnstoredEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_equippers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<ItemEquippedEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.equipment_entity)
            .collect()
    }

    fn new_unequippers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<ItemUnequippedEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.equipment_entity)
            .collect()
    }

    fn storer(&self) -> Option<EntityRef> {
        self.1
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.contains(self.0))
            .map(|(e, _)| e)
    }

    fn equipper(&self) -> Option<EntityRef> {
        self.1
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(self.0))
            .map(|(e, _)| e)
    }

    fn equipped_slots(&self) -> Option<HashSet<EquipmentSlot>> {
        self.1
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(self.0))
            .and_then(|(_, (equipment,))| equipment.get_containing_slots(self.0))
    }
}

pub trait StorageInsights {
    /// Returns true iff the given `item_entity` can be stored by this entity.
    fn can_store(&self, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being stored by this entity.
    fn is_storing(&self, item_entity: &EntityRef) -> bool;
}

impl<'a> StorageInsights for EntityInsights<'a> {
    fn can_store(&self, item_entity: &EntityRef) -> bool {
        self.1
            .select_one::<(Storage,)>(self.0)
            .map(|(storage,)| storage.get_available_slot(item_entity, self.1).is_some())
            .unwrap_or(false)
    }

    fn is_storing(&self, item_entity: &EntityRef) -> bool {
        self.1
            .select_one::<(Storage,)>(self.0)
            .map(|(storage,)| storage.contains(item_entity))
            .unwrap_or(false)
    }
}

pub trait EquipmentInsights {
    /// Returns true iff the given `item_entity` can be equipped by this entity.
    fn can_equip(&self, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being equipped by this entity.
    fn is_equipping(&self, item_entity: &EntityRef) -> bool;
}

impl<'a> EquipmentInsights for EntityInsights<'a> {
    fn can_equip(&self, item_entity: &EntityRef) -> bool {
        self.1
            .select_one::<(Equipment,)>(self.0)
            .map(|(equipment,)| equipment.get_slots_to_occupy(item_entity, self.1).is_some())
            .unwrap_or(false)
    }

    fn is_equipping(&self, item_entity: &EntityRef) -> bool {
        self.1
            .select_one::<(Equipment,)>(self.0)
            .map(|(equipment,)| equipment.contains(item_entity))
            .unwrap_or(false)
    }
}
