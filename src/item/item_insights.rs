use std::collections::HashSet;

use crate::prelude::{EntityInsights, EntityRef, EntityRefBag};

use super::{
    EntityEquippedEvt, EntityStoredEvt, EntityUnequippedEvt, EntityUnstoredEvt, Equipment,
    Equippable, Item, ItemLocation, Storage,
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
}

impl<'a> ItemInsights for EntityInsights<'a> {
    fn is_item(&self) -> bool {
        self.1.select_one::<(Item,)>(self.0).is_some()
    }

    fn location(&self) -> ItemLocation {
        if let Some((storing_entity, _)) = self
            .1
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.0.contains(self.0))
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
            .read_events::<EntityStoredEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_unstorers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<EntityUnstoredEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.storage_entity)
            .collect()
    }

    fn new_equippers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<EntityEquippedEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.equipment_entity)
            .collect()
    }

    fn new_unequippers(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<EntityUnequippedEvt>()
            .filter(|evt| &evt.entity == self.0)
            .map(|evt| evt.equipment_entity)
            .collect()
    }
}

pub trait StorageInsights {
    /// Returns true iff the given `item` entity can be stored by this entity.
    fn can_store(&self, item: &EntityRef) -> bool;
    /// Returns true iff the given `item` entity is being stored by this entity.
    fn is_storing(&self, item: &EntityRef) -> bool;
}

impl<'a> StorageInsights for EntityInsights<'a> {
    fn can_store(&self, item: &EntityRef) -> bool {
        if let Some((item,)) = self.1.select_one::<(Item,)>(item) {
            self.1
                .select_one::<(Storage,)>(self.0)
                .map(|(storage,)| storage.can_store(item))
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn is_storing(&self, item: &EntityRef) -> bool {
        self.1
            .select_one::<(Storage,)>(self.0)
            .map(|(storage,)| storage.0.contains(item))
            .unwrap_or(false)
    }
}

pub trait EquipmentInsights {
    /// Returns true iff the given `item` entity can be equipped by this entity.
    fn can_equip(&self, item: &EntityRef) -> bool;
    /// Returns true iff the given `item` entity is being equipped by this entity.
    fn is_equipping(&self, item: &EntityRef) -> bool;
}

impl<'a> EquipmentInsights for EntityInsights<'a> {
    fn can_equip(&self, item: &EntityRef) -> bool {
        if let Some((equippable,)) = self.1.select_one::<(Equippable,)>(item) {
            self.1
                .select_one::<(Equipment,)>(self.0)
                .map(|(storage,)| storage.can_equip(equippable))
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn is_equipping(&self, item: &EntityRef) -> bool {
        self.1
            .select_one::<(Equipment,)>(self.0)
            .map(|(equipment,)| equipment.contains(item))
            .unwrap_or(false)
    }
}
