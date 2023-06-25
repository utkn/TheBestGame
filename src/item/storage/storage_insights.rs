use crate::prelude::*;

use super::Storage;

pub trait StorageInsights {
    /// Returns true iff the given entity has a storage.
    fn has_storage(&self, e: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` can be stored by `storage_entity`.
    fn can_store(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool;
    /// Returns true iff the given `item_entity` is being stored by `storage_entity`.
    fn is_storing(&self, storage_entity: &EntityRef, item_entity: &EntityRef) -> bool;
}

impl<'a> StorageInsights for StateInsights<'a> {
    fn has_storage(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Storage,)>(e).is_some()
    }

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
