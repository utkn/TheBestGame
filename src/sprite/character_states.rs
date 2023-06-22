use crate::{
    physics::ProjectileGenerator,
    prelude::{EntityRefBag, InteractTarget, TargetVelocity},
    vehicle::Vehicle,
};

use super::{EntityState, EntityStateGraph};

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

pub const CHARACTER_SHOOTING: EntityState = EntityState {
    tag: "shooting",
    is_in_state: |e, state| {
        state
            .select::<(InteractTarget<ProjectileGenerator>,)>()
            .any(|(_, (p_gen_intr,))| p_gen_intr.actors.contains(e))
    },
};

pub const CHARACTER_DRIVING_AND_SHOOTING: EntityState = EntityState {
    tag: "driving_shooting",
    is_in_state: |e, state| {
        (CHARACTER_DRIVING.is_in_state)(e, state) && (CHARACTER_SHOOTING.is_in_state)(e, state)
    },
};

pub const CHARACTER_WALKING_AND_SHOOTING: EntityState = EntityState {
    tag: "walking_shooting",
    is_in_state: |e, state| {
        (CHARACTER_WALKING.is_in_state)(e, state) && (CHARACTER_SHOOTING.is_in_state)(e, state)
    },
};

pub const CHARACTER_STATE_GRAPH: EntityStateGraph = EntityStateGraph(
    "character",
    &[
        &[CHARACTER_IDLE],
        &[CHARACTER_IDLE, CHARACTER_SHOOTING],
        &[CHARACTER_IDLE, CHARACTER_WALKING],
        &[
            CHARACTER_IDLE,
            CHARACTER_WALKING,
            CHARACTER_SHOOTING,
            CHARACTER_WALKING_AND_SHOOTING,
        ],
        &[CHARACTER_IDLE, CHARACTER_DRIVING],
        &[
            CHARACTER_IDLE,
            CHARACTER_DRIVING,
            CHARACTER_SHOOTING,
            CHARACTER_DRIVING_AND_SHOOTING,
        ],
    ],
);
