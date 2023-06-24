use super::{EntityRef, State, StateCommands, Transform};

/// Represents a bundle of entities that can be generated in a single call.
pub trait EntityBundle: Sized {
    /// Constructs the bundle at the given transformation.
    fn create(trans: Transform, cmds: &mut StateCommands) -> Self;
    /// Returns the primary entity of this bundle.
    fn primary_entity(&self) -> &EntityRef;
    /// Tries to reconstruct the bundle from the primary entity.
    fn try_reconstruct(primary_entity: &EntityRef, state: &State) -> Option<Self>;
}
