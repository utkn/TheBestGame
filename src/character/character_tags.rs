use std::collections::HashSet;

use crate::{physics::ProjectileGenerator, prelude::*, vehicle::Vehicle};

use super::{Character, CharacterInsights};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterTag {
    Idle,
    Moving,
    Driving,
    Shooting,
}

impl Into<&'static str> for CharacterTag {
    fn into(self) -> &'static str {
        match self {
            CharacterTag::Idle => "idle",
            CharacterTag::Moving => "moving",
            CharacterTag::Driving => "driving",
            CharacterTag::Shooting => "shooting",
        }
    }
}

impl TagSource for Character {
    type TagType = CharacterTag;

    fn source_name() -> &'static str {
        "character"
    }

    fn try_generate(e: &EntityRef, state: &State) -> Option<HashSet<Self::TagType>> {
        if !StateInsights::of(state).is_character(e) {
            return None;
        }
        let mut tags = HashSet::new();
        let is_idle = state
            .select_one::<(TargetVelocity,)>(e)
            .map(|(target_vel,)| target_vel.x == 0. && target_vel.y == 0.)
            .unwrap_or(true);
        let is_driving = state
            .select::<(InteractTarget<Vehicle>,)>()
            .any(|(_, (vehicle_intr,))| vehicle_intr.actors.contains(e));
        let is_shooting = state
            .select::<(InteractTarget<ProjectileGenerator>,)>()
            .any(|(_, (p_gen_intr,))| p_gen_intr.actors.contains(e));
        if is_driving {
            tags.insert(CharacterTag::Driving);
        } else if is_idle {
            tags.insert(CharacterTag::Idle);
        } else {
            tags.insert(CharacterTag::Moving);
        }
        if is_shooting {
            tags.insert(CharacterTag::Shooting);
        }
        Some(tags)
    }
}
