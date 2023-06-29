use std::collections::HashMap;

use crate::prelude::*;

use super::Storage;

struct ShadowStorage(Storage);

impl From<Storage> for ShadowStorage {
    fn from(storage: Storage) -> Self {
        Self(storage)
    }
}

impl ShadowStorage {
    /// Tries to store the given entity in the underlying storage and returns `true` iff it succeeds.
    fn try_store(&mut self, item_entity: EntityRef, state: &impl StateReader) -> bool {
        // Find available slot
        if let Some(idx) = self.0.get_available_slot(&item_entity, state) {
            // Get the stack at the slot.
            if let Some(item_stack) = self.0.stacks.get_mut(idx) {
                item_stack.try_store(item_entity, state)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Tries to unstore the given entity in the underlying storage and returns `true` iff it succeeds.
    fn try_unstore(&mut self, item_entity: &EntityRef) -> bool {
        if let Some(idx) = self.0.get_containing_slot(item_entity) {
            if let Some(item_stack) = self.0.stacks.get_mut(idx) {
                item_stack.items_mut().remove(item_entity)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn take(self) -> Storage {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoreItemReq {
    pub storage_entity: EntityRef,
    pub item_entity: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct UnstoreItemReq {
    pub storage_entity: EntityRef,
    pub item_entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemStoredEvt {
    pub storage_entity: EntityRef,
    pub item_entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemUnstoredEvt {
    pub storage_entity: EntityRef,
    pub item_entity: EntityRef,
}

/// A system that handles entity storing/unstoring to/from `Storage` entities.
#[derive(Clone, Debug)]
pub struct StorageSystem;

impl<R: StateReader> System<R> for StorageSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        // Maintain the shadow storages.
        let mut shadow_storage_map = HashMap::<EntityRef, ShadowStorage>::new();
        // Perform the unstorings on the shadow storages.
        state.read_events::<UnstoreItemReq>().for_each(|evt| {
            if let Some((storage,)) = state.select_one::<(Storage,)>(&evt.storage_entity) {
                let shadow_storage = shadow_storage_map
                    .entry(evt.storage_entity)
                    .or_insert(ShadowStorage::from(storage.clone()));
                if shadow_storage.try_unstore(&evt.item_entity) {
                    cmds.emit_event(ItemUnstoredEvt {
                        storage_entity: evt.storage_entity,
                        item_entity: evt.item_entity,
                    })
                }
            }
        });
        // Perform the storings on the shadow storages.
        state.read_events::<StoreItemReq>().for_each(|evt| {
            if let Some((storage,)) = state.select_one::<(Storage,)>(&evt.storage_entity) {
                let shadow_storage = shadow_storage_map
                    .entry(evt.storage_entity)
                    .or_insert(ShadowStorage::from(storage.clone()));
                if shadow_storage.try_store(evt.item_entity, state) {
                    cmds.emit_event(ItemStoredEvt {
                        storage_entity: evt.storage_entity,
                        item_entity: evt.item_entity,
                    })
                }
            }
        });
        // Move the shadow storages into the game.
        shadow_storage_map
            .into_iter()
            .for_each(|(storage_entity, shadow_storage)| {
                cmds.set_component(&storage_entity, shadow_storage.take());
            });
        // Now, remove the invalids from all the storages.
        state.select::<(Storage,)>().for_each(|(e, _)| {
            cmds.remove_invalids::<Storage>(&e);
        })
    }
}
