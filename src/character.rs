mod character_bundle;
mod character_insights;
mod character_tags;

pub use character_bundle::*;
pub use character_insights::*;
pub use character_tags::*;

/// Represents a character in the game.
#[derive(Clone, Copy, Debug)]
pub struct Character;
