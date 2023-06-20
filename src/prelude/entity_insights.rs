use crate::prelude::{AnchorTransform, EntityRef, State};

pub struct EntityInsights<'a>(pub &'a EntityRef, pub &'a State);

impl<'a> EntityInsights<'a> {
    pub fn of(entity: &'a EntityRef, state: &'a State) -> Self {
        Self(entity, state)
    }
}

pub trait AnchoredInsights {
    fn anchor_parent(&self) -> Option<EntityRef>;
}

impl<'a> AnchoredInsights for EntityInsights<'a> {
    fn anchor_parent(&self) -> Option<EntityRef> {
        self.1
            .select_one::<(AnchorTransform,)>(self.0)
            .map(|(anchor,)| anchor.0)
    }
}
