use std::collections::HashSet;

use crate::prelude::*;

use super::Hitbox;

/// Represents insights about an entity that could possibly be a collider (i.e., have a hitbox).
pub trait ColliderInsights {
    /// Returns the entities that collide with this entity.
    fn contacts(&self) -> EntityRefSet;
    /// Returns the set of entities that just started colliding with this entity in the last update.
    fn new_collision_starters(&self) -> HashSet<EntityRef>;
    /// Returns the set of entities that just stopped colliding with this entity in the last update.
    fn new_collision_enders(&self) -> HashSet<EntityRef>;
}

impl<'a> ColliderInsights for EntityInsights<'a> {
    fn contacts(&self) -> EntityRefSet {
        self.1
            .select_one::<(InteractTarget<Hitbox>,)>(self.0)
            .map(|(coll_state,)| coll_state.actors.clone())
            .unwrap_or_default()
    }

    fn new_collision_starters(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<InteractionStartedEvt<Hitbox>>()
            .filter(|evt| &evt.target == self.0)
            .map(|evt| evt.actor)
            .collect()
    }

    fn new_collision_enders(&self) -> HashSet<EntityRef> {
        self.1
            .read_events::<InteractionEndedEvt<Hitbox>>()
            .filter(|evt| &evt.target == self.0)
            .map(|evt| evt.actor)
            .collect()
    }
}
