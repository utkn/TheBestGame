use std::collections::HashSet;

use crate::prelude::{EntityInsights, EntityRef, EntityRefBag};

use super::{
    EntityEquippedEvt, EntityStoredEvt, EntityUnequippedEvt, EntityUnstoredEvt, Equipment,
    ItemLocation, Storage,
};

pub trait ItemInsights {
    fn location(&self) -> ItemLocation;
    fn new_storers(&self) -> HashSet<EntityRef>;
    fn new_unstorers(&self) -> HashSet<EntityRef>;
    fn new_equippers(&self) -> HashSet<EntityRef>;
    fn new_unequippers(&self) -> HashSet<EntityRef>;
}

impl<'a> ItemInsights for EntityInsights<'a> {
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
