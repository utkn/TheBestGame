use crate::prelude::*;

use super::Character;

pub trait CharacterInsights {
    fn is_character(&self, e: &EntityRef) -> bool;
}

impl<'a> CharacterInsights for StateInsights<'a> {
    fn is_character(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Character,)>(e).is_some()
    }
}
