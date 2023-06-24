use std::collections::HashSet;

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct DefaultSprite;

impl TagSource for DefaultSprite {
    type TagType = &'static str;

    fn source_name() -> &'static str {
        "default"
    }

    fn try_generate(_: &EntityRef, _: &State) -> Option<HashSet<Self::TagType>> {
        Some(HashSet::new())
    }
}
