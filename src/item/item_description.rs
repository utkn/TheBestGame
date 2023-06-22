use crate::prelude::*;

use super::{Equipment, Item, Storage};

#[derive(Clone, Debug)]
pub struct ItemDescription<'a> {
    base_name: &'a Name,
    item_equipment: Option<&'a Equipment>,
    item_storage: Option<&'a Storage>,
    pub weight: f32,
}

impl<'a> PartialEq for ItemDescription<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.base_name == other.base_name
    }
}

impl<'a> ItemDescription<'a> {
    pub fn of(item: &'a EntityRef, state: &'a State) -> Option<Self> {
        let (Item(weight), base_name) = state.select_one::<(Item, Name)>(item)?;
        let item_equipment = state
            .select_one::<(Equipment,)>(item)
            .map(|(equipment,)| equipment);
        let item_storage = state
            .select_one::<(Storage,)>(item)
            .map(|(storage,)| storage);
        Some(ItemDescription {
            weight: *weight,
            base_name,
            item_equipment,
            item_storage,
        })
    }
}
