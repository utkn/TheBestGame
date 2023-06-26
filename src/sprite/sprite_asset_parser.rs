use std::collections::HashMap;

use itertools::Itertools;

use crate::prelude::TagSource;

use super::{SpriteFrames, SpriteTags};

/// Maintains the set of representible tags for a specific sprite id.
/// The representible tags are read from the names of the files in the assets folder.
#[derive(Clone, Debug, Default)]
pub struct SpriteAssetParser {
    sprite_id: String,
    // source name => all relevant SpriteFrames (e.g., idle, walking+tired, etc.)
    frames_map: HashMap<&'static str, Vec<SpriteFrames>>,
}

impl SpriteAssetParser {
    /// Creats a new representible tags instance.
    pub fn new(sprite_id: String) -> Self {
        Self {
            sprite_id,
            frames_map: Default::default(),
        }
    }

    /// Reads the corresponding assets folder to collect the representible tags for the given tag source `S`.
    pub fn with<S: TagSource>(mut self) -> Self {
        let source_name = S::source_name();
        let file_pattern = format!("./assets/{}/{}@*", self.sprite_id, source_name);
        let frames = glob::glob(&file_pattern)
            .expect("assets folder do not exist")
            .flatten()
            .flat_map(|sprite_file| SpriteFrames::load(sprite_file))
            .collect_vec();
        if frames.len() > 0 {
            self.frames_map.insert(source_name, frames);
        }
        self
    }

    /// Given the current tags of the entity, finds the sprite asset that represents it the best with respect to the tag source `S`.
    pub fn try_represent_as<'a, S: TagSource>(
        &'a self,
        existing_tags: SpriteTags<S>,
    ) -> Option<&'a SpriteFrames> {
        let src_name = S::source_name();
        let repr_tag_subsets = self.frames_map.get(src_name)?;
        let existing_tag_names = &existing_tags.into_stringified_tags();
        repr_tag_subsets
            .iter()
            // Filter only the representing tags that are subsets of the existing tags.
            .filter(|sprite_asset| sprite_asset.can_represent(existing_tag_names))
            .map(|sprite_asset| {
                (
                    sprite_asset.representation_score(existing_tag_names),
                    sprite_asset,
                )
            })
            // Get the representation that has the most number of common tags with the existing tags.
            .max_by_key(|(score, _)| *score)
            .map(|(_, repr_path)| repr_path)
    }
}
