use crate::prelude::*;

use super::Character;

pub trait CharacterInsights<'a> {
    /// Returns true iff the given entity is a character.
    fn is_character(&self, e: &EntityRef) -> bool;
}

impl<'a, R: StateReader> CharacterInsights<'a> for StateInsights<'a, R> {
    fn is_character(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Character,)>(e).is_some()
    }
}
