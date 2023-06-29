use crate::prelude::*;

use super::{Equipment, Item, Storage};

#[derive(Clone, Debug)]
pub struct ItemDescription<'a, R: StateReader> {
    base_name: &'a Name,
    item_equipment: Option<&'a Equipment>,
    item_storage: Option<&'a Storage>,
    state: &'a R,
    pub weight: f32,
}

impl<'a, R: StateReader> PartialEq for ItemDescription<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        self.base_name == other.base_name
            && match (self.item_equipment, other.item_equipment) {
                (None, Some(_)) | (Some(_), None) => false,
                (Some(eq1), Some(eq2)) => {
                    eq1.content_description(self.state) == eq2.content_description(self.state)
                }
                (None, None) => true,
            }
            && match (self.item_storage, other.item_storage) {
                (None, Some(_)) | (Some(_), None) => false,
                (None, None) => true,
                (Some(st1), Some(st2)) => {
                    st1.content_description(self.state) == st2.content_description(self.state)
                }
            }
    }
}

impl<'a, R: StateReader> ItemDescription<'a, R> {
    pub fn of(item: &'a EntityRef, state: &'a R) -> Option<Self> {
        let (Item(weight), base_name) = state.select_one::<(Item, Name)>(item)?;
        let item_equipment = state
            .select_one::<(Equipment,)>(item)
            .map(|(equipment,)| equipment);
        let item_storage = state
            .select_one::<(Storage,)>(item)
            .map(|(storage,)| storage);
        Some(ItemDescription {
            state,
            weight: *weight,
            base_name,
            item_equipment,
            item_storage,
        })
    }
}
