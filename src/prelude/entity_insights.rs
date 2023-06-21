use crate::prelude::{AnchorTransform, EntityRef, State};

/// Provides insights about a given entity with the given game system. Other modules should extend this with new functionality.
pub struct EntityInsights<'a>(pub &'a EntityRef, pub &'a State);

impl<'a> EntityInsights<'a> {
    pub fn of(entity: &'a EntityRef, state: &'a State) -> Self {
        Self(entity, state)
    }
}

/// Represents an entity that can possibly be anchored.
pub trait AnchoredInsights {
    fn anchor_parent(&self) -> Option<EntityRef>;
}

impl<'a> AnchoredInsights for EntityInsights<'a> {
    /// Returns the anchor parent if it exists.
    fn anchor_parent(&self) -> Option<EntityRef> {
        self.1
            .select_one::<(AnchorTransform,)>(self.0)
            .map(|(anchor,)| anchor.0)
    }
}
