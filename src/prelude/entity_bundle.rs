use super::EntityRef;

/// Represents a tuple of [`EntityRef`] objects.
pub trait EntityTuple<'a> {
    type AsRefTuple;
    type AsArray: IntoIterator<Item = EntityRef>;
    fn from_slice(v: &'a [EntityRef]) -> Self::AsRefTuple;
    fn into_array(self) -> Self::AsArray;
}

// Converts a type argument to the concrete type [`EntityRef`].
macro_rules! type_as_entity_ref {
    ( $t: ty ) => {
        EntityRef
    };
}

// Implement the entity tuple trait for all tuples of [`EntityRef`]s.
variadic_generics::va_expand! { ($va_len:tt) ($($va_idents:ident),+) ($($va_indices:tt),+)
    impl<'a> EntityTuple<'a> for ($(type_as_entity_ref!($va_idents),)+) {
        type AsRefTuple = ($(&'a type_as_entity_ref!($va_idents),)+);
        type AsArray = [EntityRef; $va_len];

        fn from_slice(v: &'a [EntityRef]) -> Self::AsRefTuple {
            ($(v.get($va_indices).unwrap(),)+)
        }

        fn into_array(self) -> Self::AsArray {
            [$(self.$va_indices,)+]
        }
    }

}

/// Represents a bundle of entities that can be represented as a tuple and can be generated in a single call.
pub trait EntityBundle<'a>: Sized + Clone + 'static {
    /// The corresponding tuple representation of the bundle.
    type TupleRepr: EntityTuple<'a>;
    /// Returns the unique representor of this bundle.
    fn primary_entity(&self) -> &EntityRef;
    /// Converts the concrete bundle into a tuple of entities.
    fn deconstruct(self) -> Self::TupleRepr;
    /// Converts a tuple of entities into the concrete bundle struct.
    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self;
}
