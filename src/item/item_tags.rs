use std::collections::HashSet;

use crate::prelude::*;

use super::{EquipmentSlot, Item};

#[derive(Clone, Copy, Debug, Hash)]
pub enum ItemTag {
    Ground,
    Equipped(EquipmentSlot),
    Stored,
}

impl Into<&'static str> for ItemTag {
    fn into(self) -> &'static str {
        match self {
            ItemTag::Ground => "ground",
            ItemTag::Equipped(_) => "equipped",
            ItemTag::Stored => "stored",
        }
    }
}

impl TagSource for Item {
    type TagType = ItemTag;

    fn source_name() -> &'static str {
        "item"
    }

    fn generate(_e: &EntityRef, _state: &State) -> HashSet<Self::TagType> {
        Default::default()
    }
}
