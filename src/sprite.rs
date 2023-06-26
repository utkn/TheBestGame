use std::collections::HashMap;

use crate::{building::Building, character::Character, item::Item, prelude::*, vehicle::Vehicle};

mod default_sprite;
mod representible_tags;
mod sprite_tags;

use default_sprite::*;
use representible_tags::*;
use sprite_tags::*;

#[derive(Clone, Copy, Debug)]
pub struct TilingConfig {
    pub repeat_x: u8,
    pub repeat_y: u8,
}

impl Default for TilingConfig {
    fn default() -> Self {
        Self {
            repeat_x: 1,
            repeat_y: 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub sprite_id: &'static str,
    pub z_index: usize,
    pub tiling_config: TilingConfig,
}

impl Sprite {
    pub fn new(sprite_id: &'static str, z_index: usize) -> Self {
        Self {
            sprite_id,
            z_index,
            tiling_config: Default::default(),
        }
    }

    pub fn with_tiling(mut self, repeat_x: u8, repeat_y: u8) -> Self {
        self.tiling_config.repeat_x = repeat_x;
        self.tiling_config.repeat_y = repeat_y;
        self
    }
}

fn parse_all_representible_tags_for(sprite_id: &'static str) -> RepresentibleTags {
    RepresentibleTags::new(sprite_id)
        .with::<DefaultSprite>()
        .with::<Character>()
        .with::<Vehicle>()
        .with::<Item>()
        .with::<Building>()
}

#[derive(Clone, Debug, Default)]
pub struct SpriteRepresentor {
    /// Maps a sprite id to the representible tags parsed from the relevant assets folder.
    repr_tags: HashMap<&'static str, RepresentibleTags>,
}

impl SpriteRepresentor {
    /// Tries to find the asset path that represents the given entity best with respect to the
    /// given tag source `S`.
    fn try_represent_as<'a, S: TagSource>(
        &'a mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> Option<std::path::PathBuf> {
        let sprite_id = state.select_one::<(Sprite,)>(sprite_entity)?.0.sprite_id;
        let repr_tags = self
            .repr_tags
            .entry(sprite_id)
            .or_insert_with(|| parse_all_representible_tags_for(sprite_id));
        let entity_tags = SpriteTags::<S>::of(sprite_entity, state)?;
        repr_tags.try_represent_as::<S>(entity_tags).cloned()
    }

    /// Tries to find the asset paths that represents the state of the given sprite entity the best.
    /// The returned representations are ordered with their priority.
    pub fn get_representations(
        &mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> impl Iterator<Item = std::path::PathBuf> {
        let representations = [
            self.try_represent_as::<Character>(sprite_entity, state),
            self.try_represent_as::<Vehicle>(sprite_entity, state),
            self.try_represent_as::<Item>(sprite_entity, state),
            self.try_represent_as::<Building>(sprite_entity, state),
            self.try_represent_as::<DefaultSprite>(sprite_entity, state),
        ];
        representations.into_iter().flatten()
    }
}
