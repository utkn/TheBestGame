use std::collections::HashSet;

use crate::prelude::*;

use super::{Item, ItemInsights, ItemLocation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemTag {
    Ground,
    Equipped,
    Stored,
}

impl From<ItemTag> for &'static str {
    fn from(tag: ItemTag) -> Self {
        match tag {
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

    fn try_generate(e: &EntityRef, state: &impl StateReader) -> anyhow::Result<HashSet<Self::TagType>> {
        let insights = StateInsights::of(state);
        if !insights.is_item(e) {
            anyhow::bail!("{:?} is not an item", e);
        }
        Ok(HashSet::from_iter([match insights.location_of(e) {
            ItemLocation::Ground => ItemTag::Ground,
            ItemLocation::Equipment(_) => ItemTag::Equipped,
            ItemLocation::Storage(_) => ItemTag::Stored,
        }]))
    }
}
