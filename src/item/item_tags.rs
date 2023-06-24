use std::collections::HashSet;

use crate::prelude::*;

use super::{Item, ItemInsights, ItemLocation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemTag {
    Ground,
    Equipped,
    Stored,
}

impl Into<&'static str> for ItemTag {
    fn into(self) -> &'static str {
        match self {
            ItemTag::Ground => "ground",
            ItemTag::Equipped => "equipped",
            ItemTag::Stored => "stored",
        }
    }
}

impl TagSource for Item {
    type TagType = ItemTag;

    fn source_name() -> &'static str {
        "item"
    }

    fn try_generate(e: &EntityRef, state: &State) -> Option<HashSet<Self::TagType>> {
        let insights = StateInsights::of(state);
        if !insights.is_item(e) {
            return None;
        }
        Some(HashSet::from_iter([match insights.location_of(e) {
            ItemLocation::Ground => ItemTag::Ground,
            ItemLocation::Equipment(_) => ItemTag::Equipped,
            ItemLocation::Storage(_) => ItemTag::Stored,
        }]))
    }
}
