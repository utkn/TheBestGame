use std::collections::HashSet;

use itertools::Itertools;
use notan::egui::epaint::ahash::HashMap;

use super::{
    component::{Component, ComponentIter, ComponentManager, ComponentTuple},
    event::{Event, EventManager, OptionalIter},
    EntityBundle, EntityManager, EntityRef, EntityRefBag, EntityTuple,
};

mod state_reader;

pub use state_reader::*;

#[derive(Default, Debug)]
pub struct State {
    component_mgr: ComponentManager,
    entity_mgr: EntityManager,
    event_mgr: EventManager,
    to_remove: HashSet<EntityRef>,
    bundles: HashMap<EntityRef, Vec<EntityRef>>,
}

impl State {
    /// Marks the given entity for removal.
    fn mark_for_removal(&mut self, e: &EntityRef) {
        // If the entity is part of a bundle, remove the bundle altogether.
        let containing_bundle = self
            .bundles
            .iter()
            .find(|(_, bundle_entities)| bundle_entities.contains(e));
        if let Some((&bundle_key, _)) = containing_bundle {
            // Recurse on the parent key (go up in the parent tree)
            if !self.to_remove.contains(&bundle_key) {
                return self.mark_for_removal(&bundle_key);
            }
        }
        // Otherwise, remove the entity and it's own bundle (go down)
        self.to_remove.insert(*e);
        self.bundles.get(e).cloned().map(|bundle_entities| {
            bundle_entities.into_iter().for_each(|e| {
                self.mark_for_removal(&e);
            });
        });
    }

    /// Registers a new bundle.
    fn push_bundle<'a, B: EntityBundle<'a>>(&mut self, bundle: B) {
        let bundle_key = *bundle.primary_entity();
        let bundle_vec = Vec::from_iter(bundle.deconstruct().into_array().into_iter());
        self.bundles.insert(bundle_key, bundle_vec);
    }
}

pub struct EntityRefComponentIter<'a, S: ComponentTuple<'a>>(
    ComponentIter<'a, S>,
    &'a EntityManager,
);

impl<'a, S: ComponentTuple<'a>> Iterator for EntityRefComponentIter<'a, S> {
    type Item = (EntityRef, S::RefOutput);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((id, component_tuple)) = self.0.next() {
            let version = self.1.get_curr_version(id).unwrap_or(0);
            Some((EntityRef::new(id, version), component_tuple))
        } else {
            None
        }
    }
}

impl StateReader for State {
    type EventIterator<'a, T: Event> = OptionalIter<'a, T>;
    type ComponentIterator<'a, S: ComponentTuple<'a>> = EntityRefComponentIter<'a, S>;
    /// Returns true if the given entity reference is valid.
    fn is_valid(&self, e: &EntityRef) -> bool {
        self.entity_mgr.is_valid(e)
    }

    /// Returns true if the given entity reference will be removed in the next update.
    fn will_be_removed(&self, e: &EntityRef) -> bool {
        self.to_remove.contains(e)
    }

    /// Returns an iterator over the emitted events of the given type in the last frame.
    fn read_events<'a, T: Event>(&'a self) -> Self::EventIterator<'a, T> {
        self.event_mgr.get_events_iter()
    }

    /// Returns an iterator over the components identified by the given component selector.
    fn select<'a, S: ComponentTuple<'a>>(&'a self) -> Self::ComponentIterator<'a, S> {
        EntityRefComponentIter(self.component_mgr.select::<S>(), &self.entity_mgr)
    }

    /// Returns the components of the given entity identified by the given component selector.
    fn select_one<'a, S: ComponentTuple<'a>>(
        &'a self,
        e: &EntityRef,
    ) -> Option<<S as ComponentTuple<'a>>::RefOutput> {
        if !self.is_valid(e) {
            return None;
        }
        self.component_mgr.select_one::<S>(e.id())
    }

    /// Reads a bundle of entities from the given `primary_entity`.
    fn read_bundle<'a, B: EntityBundle<'a>>(&'a self, primary_entity: &EntityRef) -> Option<B> {
        let bundle_vec = self.bundles.get(primary_entity)?;
        let bundle_tuple = B::TupleRepr::from_slice(&bundle_vec);
        let bundle = B::reconstruct(bundle_tuple);
        Some(bundle)
    }

    fn cloned_entity_manager(&self) -> EntityManager {
        self.entity_mgr.clone()
    }

    /// Updates the state through the given commands.
    fn apply_cmds(&mut self, mut cmds: StateCommands) {
        cmds.drain_modifications()
            .sorted_by_key(|m| m.0)
            .for_each(|m| m.1(self));
        // Take in the emitted events.
        self.event_mgr.merge_events(cmds.tmp_event_mgr);
    }

    /// Clears all the events in the state. Should be called at the end of an update.
    fn clear_events(&mut self) {
        self.event_mgr.clear_all()
    }

    /// Converts the entities marked as invalid to eager entity removals and copies them into the given `StateCommands`.
    fn transfer_removals(&mut self, cmds: &mut StateCommands) {
        self.to_remove.iter().for_each(|to_remove| {
            cmds.remove_entity_for_sure(to_remove);
        });
    }

    fn reset_removal_requests(&mut self) {
        self.to_remove.clear()
    }
}

/// Represents a state modification.
struct StateMod(pub u8, pub Box<dyn FnOnce(&mut State)>);

pub struct StateCommands {
    tmp_entity_mgr: EntityManager,
    tmp_event_mgr: EventManager,
    modifications: Vec<StateMod>,
}

impl<R: StateReader> From<&R> for StateCommands {
    fn from(state: &R) -> Self {
        Self {
            tmp_entity_mgr: state.cloned_entity_manager(),
            tmp_event_mgr: Default::default(),
            modifications: Default::default(),
        }
    }
}

impl StateCommands {

    /// Pushes a new event to be handled on the next update.
    pub fn emit_event<T: Event>(&mut self, evt: T) {
        if let Some(events) = self.tmp_event_mgr.get_events_mut::<T>() {
            events.push(evt);
        }
    }

    /// Dispatches a request to create a new entity in the next update and returns its would-be reference.
    /// Note that the reference will be invalid until the next update.
    pub fn create_entity(&mut self) -> EntityRef {
        let f = Box::new(|state: &mut State| {
            state.entity_mgr.create();
        });
        self.modifications.push(StateMod(0, f));
        self.tmp_entity_mgr.create()
    }

    /// Dispatches a request to create a new bundle in the next update.
    pub fn push_bundle<'a, S: EntityBundle<'a>>(&mut self, bundle: S) -> S {
        let bundle_clone = bundle.clone();
        let f = Box::new(move |state: &mut State| {
            state.push_bundle(bundle_clone);
        });
        self.modifications.push(StateMod(0, f));
        bundle
    }

    /// Dispatches a request to create a new entity with the given components in the next update.
    pub fn create_from<'a, S: ComponentTuple<'a>>(&mut self, components: S) -> EntityRef {
        let e = self.create_entity();
        self.set_components(&e, components);
        e
    }

    /// Dispatches a request to update a component on a particular entity using a closure.
    pub fn update_component<T: Component>(
        &mut self,
        e: &EntityRef,
        updater: impl FnOnce(&mut T) + 'static,
    ) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            let components = state.component_mgr.get_components_mut::<T>();
            if let Some(c) = components.get_mut(e.id()) {
                updater(c);
            }
        });
        self.modifications.push(StateMod(1, f));
    }

    /// Dispatches a request to remove the invalid references from a component.
    pub fn remove_invalids<T: EntityRefBag + Component>(&mut self, e: &EntityRef) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            let components = state.component_mgr.get_components_mut::<T>();
            if let Some(c) = components.get_mut(e.id()) {
                c.remove_invalids(&state.entity_mgr);
            }
        });
        self.modifications.push(StateMod(1, f));
    }

    /// Dispatches a request to set the component of the given entity in the next update.
    pub fn set_component<T: Component>(&mut self, e: &EntityRef, new_component: T) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            let components = state.component_mgr.get_components_mut::<T>();
            components.set(e.id(), new_component);
        });
        self.modifications.push(StateMod(1, f));
    }

    /// Dispatches a request to set the components of the given entity with the given set of components in the next update.
    pub fn set_components<'a, S: ComponentTuple<'a>>(&mut self, e: &EntityRef, components: S) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            components.insert(e.id(), &mut state.component_mgr);
        });
        self.modifications.push(StateMod(1, f));
    }

    /// Dispatches a request to remove a component from the given entity in the `next next` update.
    pub fn remove_component<T: Component>(&mut self, e: &EntityRef) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            let components = state.component_mgr.get_components_mut::<T>();
            components.remove(e.id());
        });
        self.modifications.push(StateMod(2, f));
    }

    /// Dispatches a request to remove the given entity from the system in the next next update.
    pub fn mark_for_removal(&mut self, e: &EntityRef) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            state.mark_for_removal(&e);
        });
        self.modifications.push(StateMod(3, f));
    }

    /// Returns a draining iterator on the saved modifications.
    fn drain_modifications<'a>(&'a mut self) -> impl Iterator<Item = StateMod> + 'a {
        self.modifications.drain(0..self.modifications.len())
    }

    /// Dispatches a request to remove the given entity from the system in the next update.
    fn remove_entity_for_sure(&mut self, e: &EntityRef) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            state.bundles.remove(&e);
            state.entity_mgr.remove(e.id());
            state.component_mgr.clear_components(e.id());
        });
        self.modifications.push(StateMod(3, f));
    }
}
