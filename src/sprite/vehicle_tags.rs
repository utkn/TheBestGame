use std::collections::HashSet;

use crate::{prelude::*, vehicle::Vehicle};

use super::TagSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VehicleTag {
    Driving,
    Idle,
    Damaged,
}

impl Into<&'static str> for VehicleTag {
    fn into(self) -> &'static str {
        match self {
            VehicleTag::Driving => "driving",
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

    fn generate(e: &EntityRef, state: &State) -> HashSet<Self::TagType> {
        Default::default()
    }
}
