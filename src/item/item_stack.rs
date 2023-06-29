use std::collections::HashSet;

use itertools::Itertools;

use crate::prelude::*;

use super::ItemDescription;

pub const ITEM_STACK_MAX_WEIGHT: f32 = 100.;

/// Represents a stack of item entities.
#[derive(Clone, Debug)]
pub enum ItemStack {
    /// Maximum weight this stack can hold.
    Weighted(f32, HashSet<EntityRef>),
    One(HashSet<EntityRef>),
}

impl IntoIterator for ItemStack {
    type Item = EntityRef;

    type IntoIter = <HashSet<EntityRef> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ItemStack::Weighted(_, items) | ItemStack::One(items) => items.into_iter(),
        }
    }
}

impl EntityRefBag for ItemStack {
    fn remove_invalids(&mut self, entity_mgr: &EntityManager) {
        self.items()
            .iter()
            .filter(|e| !entity_mgr.is_valid(e))
            .cloned()
            .collect_vec()
            .into_iter()
            .for_each(|e| {
                self.items_mut().remove(&e);
            });
    }
}

impl ItemStack {
    /// Creates a new weighted item stack.
    pub fn weighted() -> Self {
        Self::Weighted(ITEM_STACK_MAX_WEIGHT, Default::default())
    }

    /// Creates a new item stack that can only hold one item.
    pub fn one() -> Self {
        Self::One(Default::default())
    }

    pub fn items(&self) -> &HashSet<EntityRef> {
        match self {
            ItemStack::Weighted(_, items) | ItemStack::One(items) => items,
        }
    }

    pub fn items_mut(&mut self) -> &mut HashSet<EntityRef> {
        match self {
            ItemStack::Weighted(_, items) | ItemStack::One(items) => items,
        }
    }

    /// Returns the sum of the weights in this stack.
    pub fn total_weight(&self, state: &impl StateReader) -> f32 {
        self.items()
            .iter()
            .flat_map(|item| ItemDescription::of(item, state).map(|desc| desc.weight))
            .sum::<f32>()
    }

    /// Returns the first item on this stack.
    pub fn head_item(&self) -> Option<&EntityRef> {
        self.items().iter().next()
    }

    /// Returns the description of the first item on this stack.
    pub fn head_item_description<'a, R: StateReader>(
        &'a self,
        state: &'a R,
    ) -> Option<ItemDescription<'a, R>> {
        self.items()
            .iter()
            .next()
            .and_then(|item_entity| ItemDescription::of(item_entity, state))
    }

    /// Returns true iff the given item can be placed on this stack.
    pub fn can_store(&self, item_entity: &EntityRef, state: &impl StateReader) -> bool {
        match self {
            ItemStack::Weighted(max_weight, _) => match (
                self.head_item_description(state),
                ItemDescription::of(item_entity, state),
            ) {
                // This will be another item on the stack. The description must match with the head item in addition to the weight condition.
                (Some(d1), Some(d2)) => {
                    d1 == d2 && self.total_weight(state) + d2.weight <= *max_weight
                }
                // This will be the first item on the stack. We just need to satisfy the weight condition.
                (None, Some(d2)) => d2.weight <= *max_weight,
                (_, None) => false,
            },
            ItemStack::One(_) => self.head_item().is_none(),
        }
    }

    /// Tries to store the given entity on this stack. Returns false iff the item cannot be placed.
    pub fn try_store(&mut self, item_entity: EntityRef, state: &impl StateReader) -> bool {
        if self.can_store(&item_entity, state) {
            self.items_mut().insert(item_entity);
            true
        } else {
            false
        }
    }
}
