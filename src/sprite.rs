use std::collections::HashMap;

use crate::{building::Building, character::Character, item::Item, prelude::*, vehicle::Vehicle};

mod default_sprite;
mod sprite_asset;
mod sprite_asset_parser;
mod sprite_tags;

use default_sprite::*;
use sprite_asset::*;
use sprite_asset_parser::*;
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

#[derive(Clone, Debug)]
pub struct Sprite {
    pub sprite_id: String,
    pub z_index: usize,
    pub tiling_config: TilingConfig,
    pub curr_time: f32,
}

impl Sprite {
    pub fn new(sprite_id: impl Into<String>, z_index: usize) -> Self {
        Self {
            sprite_id: sprite_id.into(),
            z_index,
            tiling_config: Default::default(),
            curr_time: 0.,
        }
    }

    pub fn with_tiling(mut self, repeat_x: u8, repeat_y: u8) -> Self {
        self.tiling_config.repeat_x = repeat_x;
        self.tiling_config.repeat_y = repeat_y;
        self
    }
}

fn parse_all_representible_tags_for(sprite_id: String) -> SpriteAssetParser {
    SpriteAssetParser::new(sprite_id)
        .with::<DefaultSprite>()
        .with::<Character>()
        .with::<Vehicle>()
        .with::<Item>()
        .with::<Building>()
}

#[derive(Clone, Debug, Default)]
pub struct SpriteRepresentor {
    /// Maps a sprite id to the representible tags parsed from the relevant assets folder.
    repr_tags: HashMap<String, SpriteAssetParser>,
}

impl SpriteRepresentor {
    /// Tries to find the asset path that represents the given entity best with respect to the
    /// given tag source `S`.
    fn try_represent_as<'a, S: TagSource>(
        &'a mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> Option<SpriteAsset> {
        let sprite_id = &state.select_one::<(Sprite,)>(sprite_entity)?.0.sprite_id;
        let repr_tags = self
            .repr_tags
            .entry(sprite_id.clone())
            .or_insert_with(|| parse_all_representible_tags_for(sprite_id.clone()));
        let entity_tags = SpriteTags::<S>::of(sprite_entity, state)?;
        repr_tags.try_represent_as::<S>(entity_tags).cloned()
    }

    /// Tries to find the asset paths that represents the state of the given sprite entity the best.
    /// The returned representations are ordered with their priority.
    pub fn get_representations(
        &mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> impl Iterator<Item = SpriteAsset> {
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

#[derive(Clone, Copy, Debug)]
pub struct SpriteAnimationSystem;

impl System for SpriteAnimationSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.select::<(Sprite,)>().for_each(|(e, _)| {
            let dt = ctx.dt;
            cmds.update_component(&e, move |sprite: &mut Sprite| {
                sprite.curr_time += dt;
            });
        })
    }
}
