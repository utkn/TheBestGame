use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use itertools::Itertools;

use crate::prelude::TagSource;

use super::sprite_tags::SpriteTags;

pub type TagSubsets = Vec<(HashSet<String>, PathBuf)>;

/// Maintains the set of representible tags for a specific sprite id.
/// The representible tags are read from the names of the files in the assets folder.
#[derive(Clone, Debug, Default)]
pub struct RepresentibleTags {
    sprite_id: &'static str,
    // source name => tag subsets
    tag_subsets: HashMap<&'static str, TagSubsets>,
}

impl RepresentibleTags {
    /// Creats a new representible tags instance.
    pub fn new(sprite_id: &'static str) -> Self {
        Self {
            sprite_id,
            tag_subsets: Default::default(),
        }
    }

    /// Reads the corresponding assets folder to collect the representible tags for the given tag source `S`.
    pub fn with<S: TagSource>(mut self) -> Self {
        let source_name = S::source_name();
        let file_pattern = format!("./assets/{}/{}@*.png", self.sprite_id, source_name);
        let tag_subsets = glob::glob(&file_pattern)
            .expect("assets folder do not exist")
            .flatten()
            .flat_map(|tx_file| {
                let stem = tx_file
                    .file_stem()
                    .and_then(|file_stem| file_stem.to_str())
                    .map(|file_stem| file_stem.to_owned())?;
                Some((stem, tx_file))
            })
            .map(|(stem, tx_file)| {
                let stem = stem.replace(&format!("{}@", source_name), "");
                let repr_tags: HashSet<String> = stem.split("+").map(|t| t.to_owned()).collect();
                (repr_tags, tx_file)
            })
            .collect_vec();
        if tag_subsets.len() > 0 {
            self.tag_subsets.insert(source_name, tag_subsets);
        }
        self
    }

    /// Given the current tags of the entity, finds the asset path that represents it the best with respect to the tag source `S`.
    pub fn try_represent_as<'a, S: TagSource>(
        &'a self,
        existing_tags: SpriteTags<S>,
    ) -> Option<&'a std::path::PathBuf> {
        let src_name = S::source_name();
        let repr_tag_subsets = self.tag_subsets.get(src_name)?;
        let existing_tag_names = &existing_tags.into_stringified_tags();
        repr_tag_subsets
            .iter()
            // Filter only the representing tags that are subsets of the existing tags.
            .filter(|(repr_tags, _)| repr_tags.is_subset(existing_tag_names))
            .map(|(repr_tags, repr_path)| {
                // Get the number of common tags with the subset.
                let num_commons = repr_tags.intersection(existing_tag_names).count();
                (num_commons, repr_path)
            })
            // Get the representation that has the most number of common tags with the existing tags.
            .max_by_key(|(num_commons, _)| *num_commons)
            .map(|(_, repr_path)| repr_path)
    }
}
