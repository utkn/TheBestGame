use std::collections::HashSet;

use crate::prelude::*;

pub use character_states::CHARACTER_STATE_TREE;
pub use item_states::ITEM_STATE_TREE;
use itertools::Itertools;

mod character_states;
mod item_states;

#[derive(Clone, Copy, Debug)]
pub struct EntityState {
    pub tag: &'static str,
    pub is_in_state: fn(e: &EntityRef, state: &State) -> bool,
}

#[derive(Clone, Copy, Debug)]
pub struct EntityStateTree(pub &'static str, pub &'static [&'static [EntityState]]);

impl EntityStateTree {
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
        // Find the deepest branch whose leaf represents the current state of the entity.
        let deepest_branch_id = self
            .1
            .iter()
            .enumerate()
            .filter(|(_, branch)| {
                let leaf_is_good = branch
                    .last()
                    .map(|branch_leaf| (branch_leaf.is_in_state)(e, state))
                    .unwrap_or(false);
                leaf_is_good
            })
            .map(|(branch_id, branch)| (branch_id, branch.len()))
            .max_by_key(|(_, branch_depth)| *branch_depth)
            .map(|(branch_id, _)| branch_id)?;
        // Find the deepest state in the chosen branch that we can actually represent.
        let last_representible_state = self.1[deepest_branch_id]
            .iter()
            .filter(|node| available_tags.contains(&std::ffi::OsString::from(node.tag)))
            .last()?;
        Some(*last_representible_state)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite(pub EntityStateTree);

impl Sprite {
    /// Returns a path to the drawable png file that represents the given entity.
    /// e.g., {`assets_folder`}/character/idle.png
    pub fn get_file_path(
        &self,
        assets_folder: std::path::PathBuf,
        e: &EntityRef,
        state: &State,
    ) -> Option<std::path::PathBuf> {
        let mut asset_path = assets_folder;
        asset_path.push(self.0.tree_name());
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
        let deepest_state = self.0.get_deepest_state(e, state, &available_tags)?;
        asset_path.push(deepest_state.tag);
        asset_path.set_extension("png");
        Some(asset_path)
    }
}
