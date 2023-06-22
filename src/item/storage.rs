use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::prelude::*;

use super::{ItemDescription, ItemInsights, ItemLocation, ItemStack};

/// An entity that can store other entities.
#[derive(Clone, Debug)]
pub struct Storage {
    stacks: Vec<ItemStack>,
}

impl Storage {
    /// Creates a new storage with the capacity to hold `num_slots` many [`ItemStacks`].
    pub fn new(num_slots: usize) -> Self {
        let mut item_stacks = Vec::with_capacity(num_slots);
        item_stacks.resize_with(num_slots, || ItemStack::weighted());
        Self {
            stacks: item_stacks,
        }
    }

    pub fn stacks(&self) -> impl Iterator<Item = &ItemStack> {
        self.stacks.iter()
    }

    /// Returns the index of the slot in which the given `item_entity` can be stored.
    /// Returns `None` iff `item_entity` cannot be stored.
    pub fn get_available_slot(&self, item_entity: &EntityRef, state: &State) -> Option<usize> {
        self.stacks
            .iter()
            .find_position(|item_stack| item_stack.can_store(item_entity, state))
            .map(|(idx, _)| idx)
    }

    /// Returns the index of the slot in which the given `item_entity` is stored.
    /// Returns `None` iff `item_entity` is not being stored.
    pub fn get_containing_slot(&self, item_entity: &EntityRef) -> Option<usize> {
        self.stacks
            .iter()
            .find_position(|item_stack| item_stack.contains(item_entity))
            .map(|(idx, _)| idx)
    }

    pub fn content_description<'a>(&'a self, state: &'a State) -> Vec<ItemDescription<'a>> {
        self.stacks
            .iter()
            .filter_map(|item_stack| {
                if let Some(desc) = item_stack.head_item_description(state) {
                    Some(desc)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl EntityRefBag for Storage {
    fn len(&self) -> usize {
        self.stacks.iter().map(|item_stack| item_stack.len()).sum()
    }

    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef> {
        self.stacks
            .iter()
            .flat_map(|item_stack| item_stack.get_invalids(valids))
            .collect()
    }

    fn contains(&self, e: &EntityRef) -> bool {
        self.stacks.iter().any(|item_stack| item_stack.contains(e))
    }

    fn try_remove_all(&mut self, entities: &HashSet<EntityRef>) -> HashSet<EntityRef> {
        self.stacks
            .iter_mut()
            .flat_map(|item_stack| item_stack.try_remove_all(entities))
            .collect()
    }

    fn try_remove(&mut self, e: &EntityRef) -> bool {
        self.stacks
            .iter_mut()
            .any(|item_stack| item_stack.try_remove(e))
    }
}

struct ShadowStorage(Storage);

impl From<Storage> for ShadowStorage {
    fn from(storage: Storage) -> Self {
        Self(storage)
    }
}

impl ShadowStorage {
    /// Tries to store the given entity in the underlying storage and returns `true` iff it succeeds.
    fn try_store(&mut self, item_entity: EntityRef, state: &State) -> bool {
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
            if let Some(item_slot) = self.0.stacks.get_mut(idx) {
                item_slot.try_remove(item_entity)
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

/// A [`Storage`] can act as an activation/unactivation [`Interaction`].
impl Interaction for Storage {
    fn priority() -> usize {
        50
    }

    fn can_start_targeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        state.select_one::<(Storage,)>(target).is_some()
            && state.select_one::<(Character,)>(actor).is_some()
    }

    fn can_start_untargeted(actor: &EntityRef, target: &EntityRef, state: &State) -> bool {
        Self::can_start_targeted(actor, target, state)
            && EntityInsights::of(target, state).location() == ItemLocation::Ground
    }

    fn can_end_untargeted(_actor: &EntityRef, _target: &EntityRef, _state: &State) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoreItemReq {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct UnstoreItemReq {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemStoredEvt {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemUnstoredEvt {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

/// A system that handles entity storing/unstoring to/from `Storage` entities.
#[derive(Clone, Debug)]
pub struct StorageSystem;

impl System for StorageSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Maintain the shadow storages.
        let mut shadow_storage_map = HashMap::<EntityRef, ShadowStorage>::new();
        // Perform the unstorings on the shadow storages.
        state.read_events::<UnstoreItemReq>().for_each(|evt| {
            if let Some((storage,)) = state.select_one::<(Storage,)>(&evt.storage_entity) {
                let shadow_storage = shadow_storage_map
                    .entry(evt.storage_entity)
                    .or_insert(ShadowStorage::from(storage.clone()));
                if shadow_storage.try_unstore(&evt.entity) {
                    cmds.emit_event(ItemUnstoredEvt {
                        storage_entity: evt.storage_entity,
                        entity: evt.entity,
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
                if shadow_storage.try_store(evt.entity, state) {
                    cmds.emit_event(ItemStoredEvt {
                        storage_entity: evt.storage_entity,
                        entity: evt.entity,
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
            let validity_set = state.extract_validity_set();
            cmds.update_component(&e, move |storage: &mut Storage| {
                storage.remove_invalids(&validity_set);
            });
        })
    }
}
