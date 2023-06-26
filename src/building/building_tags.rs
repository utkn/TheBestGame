use std::collections::HashSet;

use crate::{
    controller::{Controller, UserInputDriver},
    physics::Hitbox,
    prelude::*,
};

use super::Building;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuildingTag {
    PlayerInside,
    PlayerOutside,
}

impl From<BuildingTag> for &'static str {
    fn from(tag: BuildingTag) -> Self {
        match tag {
            BuildingTag::PlayerInside => "inside",
            BuildingTag::PlayerOutside => "outside",
        }
    }
}

impl TagSource for Building {
    type TagType = BuildingTag;

    fn source_name() -> &'static str {
        "building"
    }

    fn try_generate(e: &EntityRef, state: &State) -> anyhow::Result<HashSet<Self::TagType>> {
        let player_inside = state
            .select_one::<(InteractTarget<Hitbox>,)>(e)
            .map(|(hb_intr,)| &hb_intr.actors)
            .map(|actors| {
                actors.iter().any(|actor| {
                    state
                        .select_one::<(Controller<UserInputDriver>,)>(actor)
                        .is_some()
                })
            })
            .unwrap_or(false);
        if player_inside {
            Ok(HashSet::from_iter([BuildingTag::PlayerInside]))
        } else {
            Ok(HashSet::from_iter([BuildingTag::PlayerOutside]))
        }
    }
}
