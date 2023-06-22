use crate::{
    prelude::{EntityRefBag, InteractTarget, TargetVelocity},
    vehicle::Vehicle,
};

use super::{EntityState, EntityStateTree};

pub const CHARACTER_IDLE: EntityState = EntityState {
    tag: "idle",
    is_in_state: |e, state| {
        state
            .select_one::<(TargetVelocity,)>(e)
            .map(|(target_vel,)| target_vel.x == 0. && target_vel.y == 0.)
            .unwrap_or(true)
    },
};

pub const CHARACTER_WALKING: EntityState = EntityState {
    tag: "walking",
    is_in_state: |e, state| !(CHARACTER_IDLE.is_in_state)(e, state),
};

pub const CHARACTER_DRIVING: EntityState = EntityState {
    tag: "driving",
    is_in_state: |e, state| {
        state
            .select::<(InteractTarget<Vehicle>,)>()
            .any(|(_, (vehicle_intr,))| vehicle_intr.actors.contains(e))
    },
};

pub const CHARACTER_STATE_TREE: EntityStateTree = EntityStateTree(
    "character",
    &[
        &[CHARACTER_IDLE],
        &[CHARACTER_IDLE, CHARACTER_WALKING],
        &[CHARACTER_IDLE, CHARACTER_DRIVING],
    ],
);
