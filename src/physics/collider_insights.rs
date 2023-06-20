use std::collections::HashSet;

use crate::prelude::*;

use super::{CollisionEndEvt, CollisionStartEvt, CollisionState};

pub trait ColliderInsights {
    fn contacts(&self) -> EntityRefSet;
    fn new_collision_starters(&self) -> HashSet<EntityRef>;
    fn new_collision_enders(&self) -> HashSet<EntityRef>;
}

impl<'a> ColliderInsights for EntityInsights<'a> {
    fn contacts(&self) -> EntityRefSet {
        self.1
            .select_one::<(CollisionState,)>(self.0)
            .map(|(coll_state,)| coll_state.colliding.clone())
            .unwrap_or_default()
    }

    fn new_collision_starters(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<CollisionStartEvt>()
            .filter(|evt| &evt.e1 == self.0)
            .map(|evt| evt.e2)
            .collect()
    }

    fn new_collision_enders(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<CollisionEndEvt>()
            .filter(|evt| &evt.e1 == self.0)
            .map(|evt| evt.e2)
            .collect()
    }
}
