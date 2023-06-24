use std::collections::HashSet;

use itertools::Itertools;

use crate::{
    item::{ItemDescription, ItemStack},
    prelude::*,
};

mod storage_activation;
mod storage_insights;
mod storage_system;

pub use storage_activation::*;
pub use storage_insights::*;
pub use storage_system::*;

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
