use std::collections::HashSet;

use crate::prelude::*;

use super::VisionField;

/// Represents insights about an entity that could possibly be seen (i.e., in a vision field) or see (i.e., has a vision field).
pub trait VisionInsights {
    /// Returns the set of entities that can see this entity.
    fn viewers(&self) -> EntityRefSet;
    /// Returns the set of entities that are being viewed by this entity.
    fn in_vision(&self) -> HashSet<EntityRef>;
}

impl<'a> VisionInsights for EntityInsights<'a> {
    fn viewers(&self) -> EntityRefSet {
        self.1
            .select_one::<(InteractTarget<VisionField>,)>(self.0)
            .map(|(intr,)| intr.actors.clone())
            .unwrap_or_default()
    }

    fn in_vision(&self) -> HashSet<EntityRef> {
        self.1
            .select::<(InteractTarget<VisionField>,)>()
            .filter(|(_, (intr,))| intr.actors.contains(self.0))
            .map(|(e, _)| e)
            .collect()
    }
}
