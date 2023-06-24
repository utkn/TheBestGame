mod character_insights;
mod character_tags;
mod create_character;

pub use character_insights::*;
pub use character_tags::*;
pub use create_character::*;

/// Represents a character in the game.
#[derive(Clone, Copy, Debug)]
pub struct Character;
