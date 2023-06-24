use std::collections::HashSet;

use crate::prelude::*;

use super::VisionField;

/// Represents insights about an entity that could possibly be seen (i.e., in a vision field) or see (i.e., has a vision field).
pub trait VisionInsights {
    /// Returns the set of entities that can see the given entity.
    fn viewers_of(&self, viewable_entity: &EntityRef) -> EntityRefSet;
    /// Returns the set of entities that are being seen by the given entity.
    fn visibles_of(&self, vision_field_entity: &EntityRef) -> HashSet<EntityRef>;
}

impl<'a> VisionInsights for StateInsights<'a> {
    fn viewers_of(&self, viewable_entity: &EntityRef) -> EntityRefSet {
        self.0
            .select_one::<(InteractTarget<VisionField>,)>(viewable_entity)
            .map(|(intr,)| intr.actors.clone())
            .unwrap_or_default()
    }

    fn visibles_of(&self, vision_field_entity: &EntityRef) -> HashSet<EntityRef> {
        self.0
            .select::<(InteractTarget<VisionField>,)>()
            .filter(|(_, (intr,))| intr.actors.contains(vision_field_entity))
            .map(|(e, _)| e)
            .collect()
    }
}
