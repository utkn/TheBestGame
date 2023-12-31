use std::collections::HashSet;

use crate::prelude::*;

use super::Hitbox;

/// Represents insights about an entity that could possibly be a collider (i.e., have a hitbox).
pub trait ColliderInsights<'a> {
    /// Returns the entities that collide with this entity.
    fn contacts_of(&self, e: &EntityRef) -> Option<&'a HashSet<EntityRef>>;
    /// Returns the concrete entities that collide with this entity.
    fn concrete_contacts_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef>;
    /// Returns the set of entities that just started colliding with this entity in the last update.
    fn new_collision_starters_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef>;
    /// Returns the set of entities that just stopped colliding with this entity in the last update.
    fn new_collision_enders_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef>;
}

impl<'a, R: StateReader> ColliderInsights<'a> for StateInsights<'a, R> {
    fn contacts_of(&self, e: &EntityRef) -> Option<&'a HashSet<EntityRef>> {
        self.0
            .select_one::<(InteractTarget<Hitbox>,)>(e)
            .map(|(coll_state,)| &coll_state.actors)
    }

    fn concrete_contacts_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef> {
        self.0
            .select_one::<(InteractTarget<Hitbox>,)>(e)
            .map(|(coll_state,)| {
                coll_state
                    .actors
                    .iter()
                    .filter(|contact| {
                        self.0
                            .select_one::<(Hitbox,)>(contact)
                            .map(|(hb,)| hb.0.is_concrete())
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn new_collision_starters_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef> {
        self.0
            .read_events::<InteractionStartedEvt<Hitbox>>()
            .filter(|evt| &evt.target == e)
            .map(|evt| &evt.actor)
            .collect()
    }

    fn new_collision_enders_of(&self, e: &EntityRef) -> HashSet<&'a EntityRef> {
        self.0
            .read_events::<InteractionEndedEvt<Hitbox>>()
            .filter(|evt| &evt.target == e)
            .map(|evt| &evt.actor)
            .collect()
    }
}
