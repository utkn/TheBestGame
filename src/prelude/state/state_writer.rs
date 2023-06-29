use crate::prelude::{Component, ComponentTuple, EntityBundle, EntityRef, EntityRefBag, Event};

/// UNUSED
pub(super) trait StateWriter {
    /// Dispatches a request to remove the given entity from the system in the next update.
    fn remove_entity_for_sure(&mut self, e: &EntityRef);
    /// Pushes a new event to be handled on the next update.
    fn emit_event<T: Event>(&mut self, evt: T);
    /// Dispatches a request to create a new entity in the next update and returns its would-be reference.
    /// Note that the reference will be invalid until the next update.
    fn create_entity(&mut self) -> EntityRef;
    /// Dispatches a request to create a new bundle in the next update.
    fn push_bundle<'a, S: EntityBundle<'a>>(&mut self, bundle: S) -> S;
    /// Dispatches a request to create a new entity with the given components in the next update.
    fn create_from<'a, S: ComponentTuple<'a>>(&mut self, components: S) -> EntityRef;
    /// Dispatches a request to update a component on a particular entity using a closure.
    fn update_component<T: Component>(
        &mut self,
        e: &EntityRef,
        updater: impl FnOnce(&mut T) + 'static,
    );
    /// Dispatches a request to remove the invalid references from a component.
    fn remove_invalids<T: EntityRefBag + Component>(&mut self, e: &EntityRef);
    /// Dispatches a request to set the component of the given entity in the next update.
    fn set_component<T: Component>(&mut self, e: &EntityRef, new_component: T);
    /// Dispatches a request to set the components of the given entity with the given set of components in the next update.
    fn set_components<'a, S: ComponentTuple<'a>>(&mut self, e: &EntityRef, components: S);
    /// Dispatches a request to remove a component from the given entity in the `next next` update.
    fn remove_component<T: Component>(&mut self, e: &EntityRef);
    /// Dispatches a request to remove the given entity from the system in the next next update.
    fn mark_for_removal(&mut self, e: &EntityRef);
}
