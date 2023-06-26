use std::collections::HashMap;

use itertools::Itertools;

/// Represents a generic bag of objects.
pub trait GenericBag: std::fmt::Debug {
    /// Returns the size of the collection.
    fn len(&self) -> usize;
    /// Returns itself as an `Any` reference, which can be used to safely cast into the underlying concrete bag.
    fn as_any(&self) -> &dyn std::any::Any;
    /// Returns itself as a mutable `Any` reference, which can be used to safely cast into the underlying concrete bag.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    /// Consumes and merges the contents of the other generic bag into this one. Should panic if the underlying concrete types are different.
    fn merge(&mut self, other: Box<dyn GenericBag>);
    /// Removes the value at the given index.
    fn remove_at(&mut self, index: usize) -> bool;
}

/// Represents a concrete bag of objects that can be stored safely as a `GenericStorage`.
pub trait ConcreteBag: 'static + GenericBag + Default {
    type Item: 'static + Clone;
}

/// Maintains a single `GenericBag` instance per implementor type.
#[derive(Default, Debug)]
pub struct GenericBagMap {
    pub(super) bags: HashMap<std::any::TypeId, Box<dyn GenericBag>>,
}

impl GenericBagMap {
    pub fn get_bag_mut<C: ConcreteBag>(&mut self) -> &mut C {
        self.bags
            .entry(std::any::TypeId::of::<C::Item>())
            .or_insert(Box::new(C::default()))
            .as_any_mut()
            .downcast_mut::<C>()
            .unwrap()
    }

    pub fn get_bag<C: ConcreteBag>(&self) -> anyhow::Result<&C> {
        self.bags
            .get(&std::any::TypeId::of::<C::Item>())
            .ok_or(anyhow::anyhow!(
                "generic bag for {:?} doesn't exist",
                std::any::type_name::<C::Item>()
            ))?
            .as_any()
            .downcast_ref::<C>()
            .ok_or(anyhow::anyhow!(
                "could not downcast the generic bag to {:?}",
                std::any::type_name::<C>()
            ))
    }

    pub fn max_len(&self) -> usize {
        self.bags
            .values()
            .map(|bag| bag.as_ref().len())
            .max()
            .unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.bags.clear()
    }

    pub fn remove_at(&mut self, index: usize) -> bool {
        self.bags
            .values_mut()
            .map(|bag| bag.remove_at(index))
            .collect_vec()
            .into_iter()
            .all(|v| v)
    }
}
