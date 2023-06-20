use std::collections::HashSet;

use crate::{
    core::{AnchorTransform, EntityRef, EntityRefBag, EntityRefSet, State},
    equipment::{EntityEquippedEvt, EntityUnequippedEvt, Equipment},
    physics::{CollisionEndEvt, CollisionEvt, CollisionStartEvt, CollisionState},
    projectile::HitEvt,
    storage::{EntityStoredEvt, EntityUnstoredEvt, Storage},
};

/// Represents the location of an entity.
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
    pub contacts: EntityRefSet,
    pub new_colliders: HashSet<EntityRef>,
    pub new_collision_starters: HashSet<EntityRef>,
    pub new_collision_enders: HashSet<EntityRef>,
    pub new_storers: HashSet<EntityRef>,
    pub new_equippers: HashSet<EntityRef>,
    pub new_unstorers: HashSet<EntityRef>,
    pub new_unequippers: HashSet<EntityRef>,
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
            contacts: state
                .select_one::<(CollisionState,)>(e)
                .map(|(coll_state,)| coll_state.colliding.clone())
                .unwrap_or_default(),
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
            new_collision_enders: state
                .read_events::<CollisionEndEvt>()
                .filter(|evt| evt.e1 == *e)
                .map(|evt| evt.e2)
                .collect(),
            new_hitters: state
                .read_events::<HitEvt>()
                .filter(|evt| evt.target == *e)
                .map(|evt| evt.hitter)
                .collect(),
            new_hit_targets: state
                .read_events::<HitEvt>()
                .filter(|evt| evt.hitter == *e)
                .map(|evt| evt.target)
                .collect(),
        }
    }
}
