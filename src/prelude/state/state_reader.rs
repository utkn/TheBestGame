use crate::prelude::{
    component_tuple::ComponentTuple, EntityBundle, EntityManager, EntityRef, Event,
};

use super::StateCommands;

pub trait StateReader: Default + 'static {
    type EventIterator<'a, T: Event>: Iterator<Item = &'a T>
    where
        Self: 'a;
    type ComponentIterator<'a, S: ComponentTuple<'a>>: Iterator<
        Item = (EntityRef, <S as ComponentTuple<'a>>::RefOutput),
    >
    where
        Self: 'a;
    /// Returns true if the given entity reference is valid.
    fn is_valid(&self, e: &EntityRef) -> bool;
    /// Returns true if the given entity reference will be removed in the next update.
    fn will_be_removed(&self, e: &EntityRef) -> bool;
    /// Returns an iterator over the emitted events of the given type in the last frame.
    fn read_events<'a, T: Event>(&'a self) -> Self::EventIterator<'a, T>;
    /// Returns an iterator over the components identified by the given component selector.
    fn select<'a, S: ComponentTuple<'a>>(&'a self) -> Self::ComponentIterator<'a, S>;
    /// Returns the components of the given entity identified by the given component selector.
    fn select_one<'a, S: ComponentTuple<'a>>(
        &'a self,
        e: &EntityRef,
    ) -> Option<<S as ComponentTuple<'a>>::RefOutput>;
    /// Reads a bundle of entities from the given `primary_entity`.
    fn read_bundle<'a, B: EntityBundle<'a>>(&'a self, primary_entity: &EntityRef) -> Option<B>;
    fn cloned_entity_manager(&self) -> EntityManager;
    fn apply_cmds(&mut self, cmds: StateCommands);
    /// Clears all the events in the state. Should be called at the end of an update.
    fn clear_events(&mut self);
    /// Converts the entities marked as invalid to eager entity removals and copies them into the given `StateCommands`.
    fn transfer_removals(&mut self, cmds: &mut StateCommands);
    fn reset_removal_requests(&mut self);
}
