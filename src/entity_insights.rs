use std::collections::HashSet;

use crate::{
    core::{AnchorTransform, EntityRef, EntityRefBag, State},
    equipment::{EntityEquippedEvt, EntityUnequippedEvt, Equipment},
    interaction::{InteractionEndedEvt, InteractionStartedEvt},
    physics::{CollisionEvt, CollisionStartEvt},
    projectile::ProjectileHitEvt,
    storage::{EntityStoredEvt, EntityUnstoredEvt, Storage},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntityLocation {
    Ground,
    Equipment(EntityRef),
    Storage(EntityRef),
}

impl EntityLocation {
    /// Returns the current location of the given entity.
    pub fn of(e: &EntityRef, state: &State) -> EntityLocation {
        if let Some((storing_entity, _)) = state
            .select::<(Storage,)>()
            .find(|(_, (storage,))| storage.0.contains(e))
        {
            EntityLocation::Storage(storing_entity)
        } else if let Some((equipping_entity, _)) = state
            .select::<(Equipment,)>()
            .find(|(_, (equipment,))| equipment.contains(e))
        {
            EntityLocation::Equipment(equipping_entity)
        } else {
            EntityLocation::Ground
        }
    }
}

/// Contains insights about an entity.
/// TODO: convert to lazy evaluation!!
#[derive(Clone, Debug)]
pub struct EntityInsights {
    pub location: EntityLocation,
    pub anchor_parent: Option<EntityRef>,
    pub new_colliders: HashSet<EntityRef>,
    pub new_collision_starters: HashSet<EntityRef>,
    pub new_storers: HashSet<EntityRef>,
    pub new_equippers: HashSet<EntityRef>,
    pub new_interactors: HashSet<EntityRef>,
    pub new_unstorers: HashSet<EntityRef>,
    pub new_unequippers: HashSet<EntityRef>,
    pub new_uninteractors: HashSet<EntityRef>,
    pub new_hitters: HashSet<EntityRef>,
    pub new_hit_targets: HashSet<EntityRef>,
}

impl EntityInsights {
    pub fn of(e: &EntityRef, state: &State) -> Self {
        let location = EntityLocation::of(e, state);
        let anchor_parent = state
            .select_one::<(AnchorTransform,)>(&e)
            .map(|(anchor,)| anchor.0);
        Self {
            location,
            anchor_parent,
            new_interactors: state
                .read_events::<InteractionStartedEvt>()
                .filter(|evt| evt.0.target == *e)
                .map(|evt| evt.0.actor)
                .collect(),
            new_storers: state
                .read_events::<EntityStoredEvt>()
                .filter(|evt| evt.entity == *e)
                .map(|evt| evt.storage_entity)
                .collect(),
            new_equippers: state
                .read_events::<EntityEquippedEvt>()
                .filter(|evt| evt.entity == *e)
                .map(|evt| evt.equipment_entity)
                .collect(),
            new_uninteractors: state
                .read_events::<InteractionEndedEvt>()
                .filter(|evt| evt.0.target == *e)
                .map(|evt| evt.0.actor)
                .collect(),
            new_unstorers: state
                .read_events::<EntityUnstoredEvt>()
                .filter(|evt| evt.entity == *e)
                .map(|evt| evt.storage_entity)
                .collect(),
            new_unequippers: state
                .read_events::<EntityUnequippedEvt>()
                .filter(|evt| evt.entity == *e)
                .map(|evt| evt.equipment_entity)
                .collect(),
            new_colliders: state
                .read_events::<CollisionEvt>()
                .filter(|evt| evt.e1 == *e)
                .map(|evt| evt.e2)
                .collect(),
            new_collision_starters: state
                .read_events::<CollisionStartEvt>()
                .filter(|evt| evt.e1 == *e)
                .map(|evt| evt.e2)
                .collect(),
            new_hitters: state
                .read_events::<ProjectileHitEvt>()
                .filter(|evt| evt.target == *e)
                .map(|evt| evt.hitter)
                .collect(),
            new_hit_targets: state
                .read_events::<ProjectileHitEvt>()
                .filter(|evt| evt.hitter == *e)
                .map(|evt| evt.target)
                .collect(),
        }
    }
}