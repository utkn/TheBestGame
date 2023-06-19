use std::collections::HashSet;

use crate::{
    activation::ActivatableComponent,
    core::{
        EntityRef, EntityRefBag, EntityRefSet, State, StateCommands, System, Transform,
        UpdateContext,
    },
    physics::CollisionState,
};

#[derive(Clone, Default, Debug)]
pub struct Storage(pub EntityRefSet);

impl Storage {
    pub fn can_store(&self, _: &EntityRef, _: &State) -> bool {
        true
    }
}

impl ActivatableComponent for Storage {
    fn can_activate(
        actor: &EntityRef,
        target: &EntityRef,
        _target_component: &Self,
        state: &State,
    ) -> bool {
        state
            .select_one::<(CollisionState,)>(actor)
            .map(|(actor_coll_state,)| actor_coll_state.colliding.contains(target))
            .unwrap_or(false)
    }

    fn activation_priority() -> usize {
        0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoreEntityReq {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug)]
pub struct UnstoreEntityReq {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityStoredEvt {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityUnstoredEvt {
    pub storage_entity: EntityRef,
    pub entity: EntityRef,
}

/// A system that handles entity storing/unstoring to/from `Storage` entities.
#[derive(Clone, Debug)]
pub struct StorageSystem;

impl System for StorageSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        // Keep a set of events to emit at the end of the execution.
        let mut unstored_events = HashSet::<EntityUnstoredEvt>::new();
        let mut stored_events = HashSet::<EntityStoredEvt>::new();
        // Remove the invalid entities from storage.
        let valids = state.extract_validity_set();
        state.select::<(Storage,)>().for_each(|(e, (storage,))| {
            // Invalidated entities should be unstored & we need to emit the appropriate event.
            let invalid_entities = storage.0.get_invalids(&valids).into_iter();
            unstored_events.extend(invalid_entities.map(|invalid_e| EntityUnstoredEvt {
                storage_entity: e,
                entity: invalid_e,
            }));
        });
        // Handle the explicit requests.
        state.read_events::<UnstoreEntityReq>().for_each(|evt| {
            // Decide whether we need to emit an event.
            if let Some((storage,)) = state.select_one::<(Storage,)>(&evt.storage_entity) {
                if storage.0.contains(&evt.entity) {
                    unstored_events.insert(EntityUnstoredEvt {
                        storage_entity: evt.storage_entity,
                        entity: evt.entity,
                    });
                }
            }
        });
        state.read_events::<StoreEntityReq>().for_each(|evt| {
            // Decide whether we need to emit an event.
            if let Some((storage,)) = state.select_one::<(Storage,)>(&evt.storage_entity) {
                if !storage.0.contains(&evt.entity) && evt.storage_entity != evt.entity {
                    stored_events.insert(EntityStoredEvt {
                        storage_entity: evt.storage_entity,
                        entity: evt.entity,
                    });
                }
            }
        });
        // Emit the events & update the state.
        unstored_events.into_iter().for_each(|evt| {
            cmds.emit_event(evt);
            cmds.update_component(&evt.storage_entity, move |storage: &mut Storage| {
                storage.0.try_remove(&evt.entity);
            });
        });
        stored_events.into_iter().for_each(|evt| {
            cmds.emit_event(evt);
            cmds.update_component(&evt.storage_entity, move |storage: &mut Storage| {
                storage.0.insert(evt.entity);
            });
            // Stored entities should not have a transform.
            cmds.remove_component::<Transform>(&evt.entity);
        });
    }
}
