use std::collections::HashSet;

use crate::{prelude::*, vehicle::Vehicle};

use super::VehicleInsights;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VehicleTag {
    Moving,
    Idle,
    Damaged,
}

impl Into<&'static str> for VehicleTag {
    fn into(self) -> &'static str {
        match self {
            VehicleTag::Moving => "moving",
            VehicleTag::Idle => "idle",
            VehicleTag::Damaged => "damaged",
        }
    }
}

impl TagSource for Vehicle {
    type TagType = VehicleTag;

    fn source_name() -> &'static str {
        "vehicle"
    }

    fn try_generate(e: &EntityRef, state: &State) -> Option<HashSet<Self::TagType>> {
        if !StateInsights::of(state).is_vehicle(e) {
            return None;
        }
        let mut tags = HashSet::new();
        let is_idle = state
            .select_one::<(TargetVelocity,)>(e)
            .map(|(target_vel,)| target_vel.x == 0. && target_vel.y == 0.)
            .unwrap_or(true);
        if is_idle {
            tags.insert(VehicleTag::Idle);
        } else {
            tags.insert(VehicleTag::Moving);
        }
        Some(tags)
    }
}
