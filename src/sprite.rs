use std::collections::HashSet;

use crate::prelude::*;

pub use character_states::CHARACTER_STATE_GRAPH;
pub use item_states::ITEM_STATE_GRAPH;
use itertools::Itertools;

mod character_states;
mod item_states;
mod prop_states;

#[derive(Clone, Copy, Debug)]
pub struct EntityState {
    pub tag: &'static str,
    pub is_state_of: fn(e: &EntityRef, state: &State) -> bool,
}

#[derive(Clone, Copy, Debug)]
pub struct EntityStateGraph(pub &'static str, pub &'static [&'static [EntityState]]);

impl EntityStateGraph {
    pub fn tree_name(&self) -> &'static str {
        self.0
    }
    /// Gets the deepest state in the state tree from the branch that represents the current state of the entity `the best`.
    pub fn get_deepest_state(
        &self,
        e: &EntityRef,
        state: &State,
        available_tags: &HashSet<std::ffi::OsString>,
    ) -> Option<EntityState> {
        // Find the deepest path from the root whose end point represents the current state of the entity.
        let deepest_path_id = self
            .1
            .iter()
            .enumerate()
            .filter(|(_, path)| {
                let leaf_is_good = path
                    .last()
                    .map(|branch_leaf| (branch_leaf.is_state_of)(e, state))
                    .unwrap_or(false);
                leaf_is_good
            })
            .map(|(path_id, path)| (path_id, path.len()))
            .max_by_key(|(_, path_depth)| *path_depth)
            .map(|(path_id, _)| path_id)?;
        // Find the deepest state in the chosen branch that we can actually represent.
        let last_representible_state = self.1[deepest_path_id]
            .iter()
            .filter(|node| available_tags.contains(&std::ffi::OsString::from(node.tag)))
            .last()?;
        Some(*last_representible_state)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite(pub &'static str, pub EntityStateGraph);

impl Sprite {
    /// Returns a path to the drawable png file that represents the given entity.
    /// e.g., {`assets_folder`}/character/{sprite_id}/idle.png
    pub fn get_file_path(
        &self,
        assets_folder: std::path::PathBuf,
        e: &EntityRef,
        state: &State,
    ) -> Option<std::path::PathBuf> {
        let mut asset_path = assets_folder;
        asset_path.push(self.1.tree_name());
        asset_path.push(self.0);
        let available_tags: HashSet<std::ffi::OsString> = std::fs::read_dir(&asset_path)
            .map(|folder| {
                let file_paths = folder.map_ok(|file| file.path()).flatten();
                file_paths
                    .filter(|file_path| {
                        let is_png = file_path
                            .extension()
                            .map(|ext| ext == "png")
                            .unwrap_or(false);
                        is_png
                    })
                    .flat_map(|file_path| file_path.file_stem().map(|os_str| os_str.to_os_string()))
                    .collect()
            })
            .ok()?;
        let deepest_state = self.1.get_deepest_state(e, state, &available_tags)?;
        asset_path.push(deepest_state.tag);
        asset_path.set_extension("png");
        Some(asset_path)
    }
}
