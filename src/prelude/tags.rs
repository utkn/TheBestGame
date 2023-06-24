use std::collections::HashSet;

use crate::prelude::*;

/// Represents a named struct that can output a set of tags for a given entity.
/// Alternatively, we can call this a `representation` of an entity.
/// For example, an idle character with opened backpack may have the tags "opened" as a `Storage` and "idle" as a `Character`.
pub trait TagSource: 'static + Clone + std::fmt::Debug {
    /// The types of the outputted tags.
    type TagType: 'static + Clone + std::fmt::Debug + std::hash::Hash + Into<&'static str>;
    /// Returns the name of this tag source.
    fn source_name() -> &'static str;
    /// Generates the set of tags for the given entity. Returns `None` if the given entity cannot be
    /// represented by this tag source.
    fn try_generate(e: &EntityRef, state: &State) -> Option<HashSet<Self::TagType>>;
}
