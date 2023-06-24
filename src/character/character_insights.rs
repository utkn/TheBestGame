use crate::prelude::*;

use super::{Character, CharacterBundle};

pub trait CharacterInsights {
    /// Returns true iff the given entity is a character.
    fn is_character(&self, e: &EntityRef) -> bool;
    /// Returns the vision field of the character entity if it exists.
    fn get_vision_field(&self, character_entity: &EntityRef) -> Option<EntityRef>;
}

impl<'a> CharacterInsights for StateInsights<'a> {
    fn is_character(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Character,)>(e).is_some()
    }

    fn get_vision_field(&self, character_entity: &EntityRef) -> Option<EntityRef> {
        CharacterBundle::try_reconstruct(character_entity, self.0).map(|cb| cb.vision_field)
    }
}
