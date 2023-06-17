use super::component::{Component, ComponentManager};

/// A tuple of `Component` objects.
pub trait ComponentTuple<'a>: Clone + 'static {
    type RefOutput;
    /// Returns true iff the manager contains an entity with these specific set of components.
    fn matches(entity_id: usize, mgr: &'a ComponentManager) -> bool;
    /// Materializes the components associated with the given entity.
    fn try_fetch(entity_id: usize, mgr: &'a ComponentManager) -> Option<Self::RefOutput>;
    /// Batch adds the components to the given entity.
    fn insert(self, entity_id: usize, mgr: &mut ComponentManager);
}

// Implement the component tuple trait for all tuples of components.
variadic_generics::va_expand! { ($va_len:tt) ($($va_idents:ident),+) ($($va_indices:tt),+)
    impl<'a, $($va_idents: Component),+> ComponentTuple<'a> for ($($va_idents,)+) {
        type RefOutput = ($(&'a $va_idents,)+);

        fn matches(entity_id: usize, mgr: &ComponentManager) -> bool {
            $(mgr.get_components::<$va_idents>().map(|bag| bag.has(entity_id)).unwrap_or(false))&&+
        }

        fn try_fetch(entity_id: usize, mgr: &'a ComponentManager) -> Option<Self::RefOutput> {
            let out = (
                $(mgr.get_components::<$va_idents>()
                    .map(|bag| bag.get(entity_id))
                    .flatten()?,)
                +);
            Some(out)
        }

        fn insert(self, entity_id: usize, mgr: &mut ComponentManager) {
            $(mgr.get_components_mut::<$va_idents>().set(entity_id, self.$va_indices));+
        }
    }
}
