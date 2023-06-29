use crate::prelude::*;

use super::Transform;

/// Provides insights about the given state of the system. Other modules should extend this with new functionality.
pub struct StateInsights<'a, R: StateReader>(pub &'a R);

impl<'a, R: StateReader> StateInsights<'a, R> {
    pub fn of(state: &'a R) -> Self {
        Self(state)
    }
}

/// Provides insights about entities that can possibly be anchored.
pub trait AnchoredInsights<'a> {
    fn anchor_parent_of(&self, e: &EntityRef) -> Option<&'a EntityRef>;
}

impl<'a, R: StateReader> AnchoredInsights<'a> for StateInsights<'a, R> {
    /// Returns the anchor parent if it exists.
    fn anchor_parent_of(&self, e: &EntityRef) -> Option<&'a EntityRef> {
        self.0
            .select_one::<(AnchorTransform,)>(e)
            .map(|(anchor,)| &anchor.0)
    }
}

/// Provides insights about entities that can possibly have a transform.
pub trait TransformInsights<'a> {
    /// Returns the transform of the given entity.
    fn transform_of(&self, e: &EntityRef) -> Option<&'a Transform>;
    /// Returns the distance square between given two entities.
    fn dist_sq_between(&self, e1: &EntityRef, e2: &EntityRef) -> Option<f32>;
    /// Returns the position difference e1 - e2.
    fn pos_diff(&self, e1: &EntityRef, e2: &EntityRef) -> Option<(f32, f32)>;
}

impl<'a, R: StateReader> TransformInsights<'a> for StateInsights<'a, R> {
    fn transform_of(&self, e: &EntityRef) -> Option<&'a Transform> {
        self.0.select_one::<(Transform,)>(e).map(|(trans,)| trans)
    }

    fn dist_sq_between(&self, e1: &EntityRef, e2: &EntityRef) -> Option<f32> {
        let t1 = self.transform_of(e1)?;
        let t2 = self.transform_of(e2)?;
        let (dx, dy) = (t2.x - t1.x, t2.y - t1.y);
        Some(dx * dx + dy * dy)
    }

    fn pos_diff(&self, e1: &EntityRef, e2: &EntityRef) -> Option<(f32, f32)> {
        let t1 = self.transform_of(e1)?;
        let t2 = self.transform_of(e2)?;
        Some((t1.x - t2.x, t1.y - t2.y))
    }
}
