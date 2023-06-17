use super::generic_bag::{ConcreteBag, GenericBag, GenericBagMap};

pub use super::component_tuple::ComponentTuple;

pub trait Component: Clone + std::fmt::Debug + 'static {}
impl<T> Component for T where T: Clone + std::fmt::Debug + 'static {}

/// A vector of components stored contigously in the memory.
#[derive(Clone, Debug)]
pub struct ComponentVec<T>(Vec<Option<T>>);

impl<T: Component> Default for ComponentVec<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<T: Component> GenericBag for ComponentVec<T> {
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
        let other_component_vec = other
            .as_any_mut()
            .downcast_mut::<ComponentVec<T>>()
            .unwrap();
        self.0
            .extend(other_component_vec.0.drain(..other_component_vec.0.len()));
    }
}

impl<T: Component> ConcreteBag for ComponentVec<T> {
    type Item = T;
}

impl<T: Component> ComponentVec<T> {
    pub fn get(&self, id: usize) -> Option<&T> {
        self.0.get(id).map(|opt_c| opt_c.as_ref()).flatten()
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.0.get_mut(id).map(|opt_c| opt_c.as_mut()).flatten()
    }

    pub fn has(&self, id: usize) -> bool {
        self.get(id).is_some()
    }

    pub fn set(&mut self, id: usize, c: T) {
        if id >= self.0.len() {
            self.0.resize(id + 1, None);
        }
        self.0.get_mut(id).map(|opt_c| opt_c.insert(c));
    }

    pub fn remove(&mut self, id: usize) -> Option<T> {
        self.0.get_mut(id).map(|opt_c| opt_c.take()).flatten()
    }
}

/// Manages multiple types of components associated with entities.
#[derive(Default, Debug)]
pub struct ComponentManager(GenericBagMap);

impl ComponentManager {
    pub fn get_components_mut<T>(&mut self) -> &mut ComponentVec<T>
    where
        T: Component,
    {
        self.0.get_bag_mut::<ComponentVec<T>>()
    }

    pub fn get_components<T>(&self) -> Option<&ComponentVec<T>>
    where
        T: Component,
    {
        self.0.get_bag::<ComponentVec<T>>()
    }

    /// Fetches all the components as a tuple.
    pub fn select<'a, S: ComponentTuple<'a>>(
        &'a self,
    ) -> impl Iterator<Item = (usize, <S as ComponentTuple<'a>>::RefOutput)> {
        (0..self.0.max_len())
            .filter(|id| S::matches(*id, self))
            .map(|id| (id, S::try_fetch(id, self).unwrap()))
    }

    /// Fetches the component tuple associated with the given entity.
    pub fn select_one<'a, S: ComponentTuple<'a>>(
        &'a self,
        id: usize,
    ) -> Option<<S as ComponentTuple<'a>>::RefOutput> {
        S::try_fetch(id, self)
    }
}
