mod character_tags;
mod tags;
mod vehicle_tags;

use std::collections::HashMap;

pub use tags::TagSource;

use crate::{prelude::*, vehicle::Vehicle};

use self::tags::{ExistingTags, RepresentibleTags};

#[derive(Clone, Copy, Debug)]
pub struct Sprite(pub &'static str);

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
        let sprite_id = state.select_one::<(Sprite,)>(sprite_entity)?.0 .0;
        let repr_tags = self
            .repr_tags
            .entry(sprite_id)
            .or_insert_with(|| parse_all_representible_tags_for(sprite_id));
        let existing_tags = ExistingTags::<S>::of(sprite_id, sprite_entity, state);
        repr_tags.try_represent_as::<S>(&existing_tags).cloned()
    }
}

fn parse_all_representible_tags_for(sprite_id: &'static str) -> RepresentibleTags {
    RepresentibleTags::new(sprite_id)
        .with::<Character>()
        .with::<Vehicle>()
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
        ];
        representations.into_iter().flatten()
    }
}
