use std::collections::HashMap;

use crate::{character::Character, item::Item, prelude::*, vehicle::Vehicle};

mod default_sprite;

use default_sprite::*;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub sprite_id: &'static str,
    pub z_index: usize,
}

impl Sprite {
    pub fn new(sprite_id: &'static str, z_index: usize) -> Self {
        Self { sprite_id, z_index }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpriteRepresentor {
    repr_tags: HashMap<&'static str, RepresentibleTags>,
}

impl SpriteRepresentor {
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
        let entity_tags = EntityTags::<S>::of(sprite_id, sprite_entity, state)?;
        repr_tags.try_represent_as::<S>(&entity_tags).cloned()
    }
}

fn parse_all_representible_tags_for(sprite_id: &'static str) -> RepresentibleTags {
    RepresentibleTags::new(sprite_id)
        .with::<DefaultSprite>()
        .with::<Character>()
        .with::<Vehicle>()
        .with::<Item>()
}

impl SpriteRepresentor {
    pub fn represent(
        &mut self,
        sprite_entity: &EntityRef,
        state: &State,
    ) -> impl Iterator<Item = std::path::PathBuf> {
        let representations = [
            self.try_represent_as::<Character>(sprite_entity, state),
            self.try_represent_as::<Vehicle>(sprite_entity, state),
            self.try_represent_as::<Item>(sprite_entity, state),
            self.try_represent_as::<DefaultSprite>(sprite_entity, state),
        ];
        representations.into_iter().flatten()
    }
}
