use std::collections::HashMap;

use crate::{building::Building, character::Character, item::Item, prelude::*, vehicle::Vehicle};

mod default_sprite;
mod sprite_asset_parser;
mod sprite_frames;
mod sprite_tags;

use default_sprite::*;
use sprite_asset_parser::*;
use sprite_frames::*;
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

#[derive(Clone, Debug, Default)]
pub struct SpriteRepresentor {
    /// Maps a sprite id to the representible tags parsed from the relevant assets folder.
    repr_tags: HashMap<String, SpriteAssetParser>,
}

impl SpriteRepresentor {
    /// Parses the sprite asset folder for the given sprite id.
    fn parse_for_sprite_id(&mut self, sprite_entity: &EntityRef, state: &State) -> Option<()> {
        let sprite_id = &state.select_one::<(Sprite,)>(sprite_entity)?.0.sprite_id;
        self.repr_tags.entry(sprite_id.clone()).or_insert_with(|| {
            let sprite_id = sprite_id.clone();
            SpriteAssetParser::new(sprite_id)
                .with::<DefaultSprite>()
                .with::<Character>()
                .with::<Vehicle>()
                .with::<Item>()
                .with::<Building>()
        });
        Some(())
    }
    /// Tries to find the asset path that represents the given entity best with respect to the
    /// given tag source `S`.
    fn try_represent_as<'a, S: TagSource>(
        &'a self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> anyhow::Result<&'a SpriteFrames> {
        let entity_tags = SpriteTags::<S>::of(sprite_entity, state)?;
        let sprite_id = &state
            .select_one::<(Sprite,)>(sprite_entity)
            .ok_or(anyhow::anyhow!(
                "{:?} is not a sprite entity",
                sprite_entity
            ))?
            .0
            .sprite_id;
        let repr_tags = self.repr_tags.get(sprite_id).ok_or(anyhow::anyhow!(
            "no representible tags were loaded for the sprite id {:?}",
            sprite_id
        ))?;
        repr_tags
            .try_represent_as::<S>(entity_tags.clone())
            .ok_or(anyhow::anyhow!(
                "no possible representation for the sprite entity {:?} with tags {:?}",
                sprite_entity,
                &entity_tags
            ))
    }

    /// Tries to find the asset paths that represents the state of the given sprite entity the best.
    /// The returned representations are ordered with their priority.
    pub fn get_representations<'a>(
        &'a mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> impl Iterator<Item = &'a SpriteFrames> {
        self.parse_for_sprite_id(sprite_entity, state);
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
