use std::collections::HashSet;

use crate::{
    core::{AnchorTransform, EntityRef, EntityRefBag, State},
    equipment::{EntityEquippedEvt, EntityUnequippedEvt, Equipment},
    interaction::{InteractionEndedEvt, InteractionStartedEvt},
    physics::{CollisionEvt, CollisionStartEvt},
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

#[derive(Clone, Debug)]
pub struct EntityInsights {
    pub location: EntityLocation,
    pub colliders: HashSet<EntityRef>,
    pub collision_starters: HashSet<EntityRef>,
    pub anchor_parent: Option<EntityRef>,
    pub storers: HashSet<EntityRef>,
    pub equippers: HashSet<EntityRef>,
    pub interactors: HashSet<EntityRef>,
    pub unstorers: HashSet<EntityRef>,
    pub unequippers: HashSet<EntityRef>,
    pub uninteractors: HashSet<EntityRef>,
}

impl EntityInsights {
    pub fn of(e: &EntityRef, state: &State) -> Self {
        let location = EntityLocation::of(e, state);
        let colliders = state
            .read_events::<CollisionEvt>()
            .filter(|evt| evt.e1 == *e)
            .map(|evt| evt.e2)
            .collect();
        let collision_starters = state
            .read_events::<CollisionStartEvt>()
            .filter(|evt| evt.e1 == *e)
            .map(|evt| evt.e2)
            .collect();
        let anchor_parent = state
            .select_one::<(AnchorTransform,)>(&e)
            .map(|(anchor,)| anchor.0);
        let storers = state
            .read_events::<EntityStoredEvt>()
            .filter(|evt| evt.entity == *e)
            .map(|evt| evt.storage_entity)
            .collect();
        let equippers = state
            .read_events::<EntityEquippedEvt>()
            .filter(|evt| evt.entity == *e)
            .map(|evt| evt.equipment_entity)
            .collect();
        let unstorers = state
            .read_events::<EntityUnstoredEvt>()
            .filter(|evt| evt.entity == *e)
            .map(|evt| evt.storage_entity)
            .collect();
        let unequippers = state
            .read_events::<EntityUnequippedEvt>()
            .filter(|evt| evt.entity == *e)
            .map(|evt| evt.equipment_entity)
            .collect();
        let interactors = state
            .read_events::<InteractionStartedEvt>()
            .filter(|evt| evt.0.target == *e)
            .map(|evt| evt.0.actor)
            .collect();
        let uninteractors = state
            .read_events::<InteractionEndedEvt>()
            .filter(|evt| evt.0.target == *e)
            .map(|evt| evt.0.actor)
            .collect();
        Self {
            interactors,
            storers,
            equippers,
            uninteractors,
            unstorers,
            unequippers,
            location,
            colliders,
            collision_starters,
            anchor_parent,
        }
    }
}
