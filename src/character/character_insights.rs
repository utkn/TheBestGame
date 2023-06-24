use std::collections::HashSet;

use crate::{
    physics::{VisionField, VisionInsights},
    prelude::*,
};

use super::Character;

pub trait CharacterInsights {
    /// Returns true iff the given entity is a character.
    fn is_character(&self, e: &EntityRef) -> bool;
    /// Returns the vision field of the character entity if it exists.
    fn vision_field_of(&self, character_entity: &EntityRef) -> Option<EntityRef>;
    /// Returns the entities that are visible by the given character.
    fn visibles_of_character(&self, character_entity: &EntityRef) -> Option<HashSet<EntityRef>>;
    /// Returns true iff `character_entity` can see the given `target`.
    fn can_character_see(&self, character_entity: &EntityRef, target: &EntityRef) -> bool;
}

impl<'a> CharacterInsights for StateInsights<'a> {
    fn is_character(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Character,)>(e).is_some()
    }

    fn vision_field_of(&self, character_entity: &EntityRef) -> Option<EntityRef> {
        self.0
            .select::<(AnchorTransform, VisionField)>()
            .find(|(_, (anchor, _))| &anchor.0 == character_entity)
            .map(|(e, _)| e)
    }

    fn visibles_of_character(&self, character_entity: &EntityRef) -> Option<HashSet<EntityRef>> {
        let vision_field = self.vision_field_of(character_entity)?;
        Some(self.visibles_of(&vision_field))
    }

    fn can_character_see(&self, character_entity: &EntityRef, target: &EntityRef) -> bool {
        self.visibles_of_character(character_entity)
            .map(|visibles| visibles.contains(target))
            .unwrap_or(false)
    }
}
