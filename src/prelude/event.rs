use std::collections::HashSet;

use super::generic_bag::{ConcreteBag, GenericBag, GenericBagMap};

pub trait Event: Clone + std::fmt::Debug + 'static {}
impl<T> Event for T where T: Clone + std::fmt::Debug + 'static {}

/// A vector of events stored contigously in the memory.
#[derive(Clone, Debug)]
pub(super) struct EventVec<T>(Vec<T>);

impl<T: Event> Default for EventVec<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<T: Event> GenericBag for EventVec<T> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn merge(&mut self, mut other: Box<dyn GenericBag>) {
        let other_event_vec = other.as_any_mut().downcast_mut::<EventVec<T>>().unwrap();
        self.0
            .extend(other_event_vec.0.drain(..other_event_vec.0.len()));
    }

    fn remove_at(&mut self, index: usize) -> bool {
        if index >= self.len() {
            return false;
        }
        self.0.remove(index);
        true
    }
}

impl<T: Event> ConcreteBag for EventVec<T> {
    type Item = T;
}

impl<T: Event> EventVec<T> {
    /// Pushes a new event to this event vector.
    pub(super) fn push(&mut self, evt: T) {
        self.0.push(evt)
    }

    /// Returns an iterator over the elements of this event vector.
    pub(super) fn iter<'a>(&'a self) -> core::slice::Iter<'a, T> {
        self.0.iter()
    }
}

/// An iterator that can initialized with `None` to be empty.
pub struct OptionalIter<'a, T>(Option<core::slice::Iter<'a, T>>);

impl<'a, T> Iterator for OptionalIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = &mut self.0 {
            it.next()
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
pub(super) struct EventManager(GenericBagMap);

impl EventManager {
    pub(super) fn clear_all(&mut self) {
        self.0.clear()
    }

    pub(super) fn merge_events(&mut self, mut other: EventManager) {
        let my_type_ids: HashSet<std::any::TypeId> = self.0.bags.keys().cloned().collect();
        let other_type_ids: HashSet<std::any::TypeId> = other.0.bags.keys().cloned().collect();
        let to_clone = other_type_ids.difference(&my_type_ids);
        self.0
            .bags
            .extend(to_clone.map(|tid| (*tid, other.0.bags.remove(tid).unwrap())));
        let to_merge = other_type_ids.intersection(&my_type_ids);
        to_merge.for_each(|tid| {
            let other_bag = other.0.bags.remove(tid).unwrap();
            self.0.bags.get_mut(tid).unwrap().merge(other_bag);
        });
    }

    pub(super) fn get_events_mut<T: Event>(&mut self) -> Option<&mut EventVec<T>> {
        self.0.get_bag_mut::<EventVec<T>>().ok()
    }

    pub(super) fn get_events<T: Event>(&self) -> Option<&EventVec<T>> {
        self.0.get_bag::<EventVec<T>>().ok()
    }

    pub(super) fn get_events_iter<'a, T: Event>(&'a self) -> OptionalIter<'a, T> {
        if let Some(evts) = self.get_events() {
            OptionalIter(Some(evts.iter()))
        } else {
            OptionalIter(None)
        }
    }
}
