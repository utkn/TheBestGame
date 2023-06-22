use std::collections::HashSet;

use crate::prelude::*;

use super::ItemDescription;

pub const ITEM_STACK_MAX_WEIGHT: f32 = 100.;

#[derive(Clone, Debug)]
pub struct ItemStack {
    max_weight: f32,
    items: EntityRefSet,
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::new(ITEM_STACK_MAX_WEIGHT)
    }
}

impl EntityRefBag for ItemStack {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef> {
        self.items.get_invalids(valids)
    }

    fn contains(&self, e: &EntityRef) -> bool {
        self.items.contains(e)
    }

    fn try_remove_all(&mut self, entities: &HashSet<EntityRef>) -> HashSet<EntityRef> {
        self.items.try_remove_all(entities)
    }

    fn try_remove(&mut self, e: &EntityRef) -> bool {
        self.items.try_remove(e)
    }
}

impl ItemStack {
    pub fn new(max_weight: f32) -> Self {
        Self {
            max_weight,
            items: Default::default(),
        }
    }

    fn total_weight(&self, state: &State) -> f32 {
        self.items
            .iter()
            .flat_map(|item| ItemDescription::of(item, state).map(|desc| desc.weight))
            .sum::<f32>()
    }

    pub fn head_item(&self) -> Option<&EntityRef> {
        self.items.iter().next()
    }

    pub fn head_item_description<'a>(&'a self, state: &'a State) -> Option<ItemDescription<'a>> {
        self.items
            .iter()
            .next()
            .and_then(|item_entity| ItemDescription::of(item_entity, state))
    }

    pub fn can_store(&self, item_entity: &EntityRef, state: &State) -> bool {
        match (
            self.head_item_description(state),
            ItemDescription::of(item_entity, state),
        ) {
            (Some(d1), Some(d2)) => {
                d1 == d2 && self.total_weight(state) + d2.weight <= self.max_weight
            }
            (None, Some(_)) => true,
            (_, None) => false,
        }
    }

    pub fn force_store(&mut self, item_entity: EntityRef) {
        self.items.insert(item_entity);
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityRef> {
        self.items.iter()
    }
}
