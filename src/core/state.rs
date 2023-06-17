use itertools::Itertools;

use super::{
    component::{Component, ComponentManager, ComponentTuple},
    event::{Event, EventManager},
    EntityManager, EntityRef, EntityValiditySet,
};

#[derive(Default, Debug)]
pub struct State {
    component_mgr: ComponentManager,
    entity_mgr: EntityManager,
    event_mgr: EventManager,
}

impl State {
    /// Clears all the events in the state. Should be called at the end of an update.
    pub fn clear_events(&mut self) {
        self.event_mgr.clear_all()
    }

    /// Updates the state through the given commands.
    pub fn apply_cmds(&mut self, mut cmds: StateCommands) {
        cmds.drain_modifications()
            .sorted_by_key(|m| m.0)
            .for_each(|m| m.1(self));
        // Take in the emitted events.
        self.event_mgr.merge_events(cmds.tmp_event_mgr);
    }

    /// Returns true if the given entity reference is valid.
    pub fn is_valid(&self, e: &EntityRef) -> bool {
        self.entity_mgr.is_valid(e)
    }

    /// Returns an iterator over the emitted events of the given type in the last frame.
    pub fn read_events<'a, T: Event>(&'a self) -> impl Iterator<Item = &'a T> {
        self.event_mgr.get_events_iter()
    }

    /// Returns the set of valid entities.
    pub fn extract_validity_set(&self) -> EntityValiditySet {
        self.entity_mgr.extract_validity_set()
    }

    /// Returns an iterator over the components identified by the given component selector.
    pub fn select<'a, S: ComponentTuple<'a>>(
        &'a self,
    ) -> impl Iterator<Item = (EntityRef, <S as ComponentTuple<'a>>::RefOutput)> {
        self.component_mgr.select::<S>().map(|(id, res)| {
            let version = self.entity_mgr.get_curr_version(id).unwrap_or(0);
            (EntityRef::new(id, version), res)
        })
    }

    /// Returns the components of the given entity identified by the given component selector.
    pub fn select_one<'a, S: ComponentTuple<'a>>(
        &'a self,
        e: &EntityRef,
    ) -> Option<<S as ComponentTuple<'a>>::RefOutput> {
        if !self.is_valid(e) {
            return None;
        }
        self.component_mgr.select_one::<S>(e.id())
    }
}

pub struct StateMod(pub u8, pub Box<dyn FnOnce(&mut State)>);

pub struct StateCommands {
    tmp_entity_mgr: EntityManager,
    tmp_event_mgr: EventManager,
    modifications: Vec<StateMod>,
}

impl From<&State> for StateCommands {
    fn from(state: &State) -> Self {
        Self {
            tmp_entity_mgr: state.entity_mgr.clone(),
            tmp_event_mgr: Default::default(),
            modifications: Default::default(),
        }
    }
}

impl StateCommands {
    /// Returns a draining filter on the saved modifications.
    pub(super) fn drain_modifications<'a>(&'a mut self) -> impl Iterator<Item = StateMod> + 'a {
        self.modifications.drain(0..self.modifications.len())
    }

    /// Pushes a new event to be handled on the next update.
    pub fn emit_event<T: Event>(&mut self, evt: T) {
        self.tmp_event_mgr.get_events_mut::<T>().push(evt)
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

    /// Dispatches a request to remove a component from the given entity in the next update.
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

    /// Dispatches a request to remove the given entity from the system in the next update.
    pub fn remove_entity(&mut self, e: &EntityRef) {
        let e = *e;
        let f = Box::new(move |state: &mut State| {
            if !state.is_valid(&e) {
                return;
            }
            state.entity_mgr.remove(e.id());
            state.component_mgr.clear_components(e.id());
        });
        self.modifications.push(StateMod(3, f));
    }
}
