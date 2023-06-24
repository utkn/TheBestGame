use crate::prelude::*;

use super::Storage;

pub trait StorageInsights {
    /// Returns true iff the given entity has a storage.
    fn has_storage(&self, e: &EntityRef) -> bool;
}

impl<'a> StorageInsights for StateInsights<'a> {
    fn has_storage(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Storage,)>(e).is_some()
    }
}
