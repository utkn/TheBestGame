use crate::prelude::{AnchorTransform, EntityRef, State};

/// Provides insights about the given state of the system. Other modules should extend this with new functionality.
pub struct StateInsights<'a>(pub &'a State);

impl<'a> StateInsights<'a> {
    pub fn of(state: &'a State) -> Self {
        Self(state)
    }
}

/// Represents an entity that can possibly be anchored.
pub trait AnchoredInsights {
    fn anchor_parent_of(&self, e: &EntityRef) -> Option<EntityRef>;
}

impl<'a> AnchoredInsights for StateInsights<'a> {
    /// Returns the anchor parent if it exists.
    fn anchor_parent_of(&self, e: &EntityRef) -> Option<EntityRef> {
        self.0
            .select_one::<(AnchorTransform,)>(e)
            .map(|(anchor,)| anchor.0)
    }
}
