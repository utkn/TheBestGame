use std::collections::HashSet;

use crate::{prelude::*, vehicle::Vehicle};

use super::VehicleInsights;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VehicleTag {
    Moving,
    Idle,
    Damaged,
}

impl From<VehicleTag> for &'static str {
    fn from(tag: VehicleTag) -> Self {
        match tag {
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

    fn try_generate(e: &EntityRef, state: &impl StateReader) -> anyhow::Result<HashSet<Self::TagType>> {
        if !StateInsights::of(state).is_vehicle(e) {
            anyhow::bail!("{:?} is not a vehicle", e);
        }
        let mut tags = HashSet::new();
        let is_idle = state
            .select_one::<(Velocity,)>(e)
            .map(|(vel,)| vel.x == 0. && vel.y == 0.)
            .unwrap_or(true);
        if is_idle {
            tags.insert(VehicleTag::Idle);
        } else {
            tags.insert(VehicleTag::Moving);
        }
        Ok(tags)
    }
}
