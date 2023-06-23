use crate::{
    item::Storage,
    prelude::{EntityRefBag, InteractTarget},
    vehicle::Vehicle,
};

use super::{EntityStateGraph, TagGenerator};

pub const DEFAULT: TagGenerator = TagGenerator {
    tag: "default",
    is_state_of: |_, _| true,
};

pub const STORAGE_ACTIVE: TagGenerator = TagGenerator {
    tag: "storage_active",
    is_state_of: |e, state| {
        state
            .select_one::<(InteractTarget<Storage>,)>(e)
            .map(|(storage_intr,)| storage_intr.actors.len() > 0)
            .unwrap_or(false)
    },
};

pub const VEHICLE_ACTIVE: TagGenerator = TagGenerator {
    tag: "vehicle_active",
    is_state_of: |e, state| {
        state
            .select_one::<(InteractTarget<Vehicle>,)>(e)
            .map(|(storage_intr,)| storage_intr.actors.len() > 0)
            .unwrap_or(false)
    },
};

pub const PROP_STATE_GRAPH: EntityStateGraph = EntityStateGraph(
    "prop",
    &[
        &[DEFAULT],
        &[DEFAULT, STORAGE_ACTIVE],
        &[DEFAULT, VEHICLE_ACTIVE],
    ],
);
