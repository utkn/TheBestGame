use crate::prelude::*;

use super::Storage;

pub trait StorageInsights {
    fn is_storage(&self, e: &EntityRef) -> bool;
}

impl<'a> StorageInsights for StateInsights<'a> {
    fn is_storage(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Storage,)>(e).is_some()
    }
}
