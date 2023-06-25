use crate::{
    character::CharacterInsights, physics::ProjectileInsights, prelude::*, vehicle::VehicleInsights,
};

use super::{AiTask, AiTaskOutput};

pub(super) fn try_get_enemy_on_sight(actor: &EntityRef, state: &State) -> Option<EntityRef> {
    let insights = StateInsights::of(state);
    let visibles = insights.visibles_of_character(actor).unwrap_or_default();
    let target = visibles.into_iter().find(|e| {
        insights.is_character(e)
            || (insights.is_vehicle(e)
                && insights
                    .driver_of(e)
                    .map(|driver| insights.is_character(driver))
                    .unwrap_or(false))
    });
    target
}

pub(super) fn try_move_towards_projectile(actor: &EntityRef, state: &State) -> Option<(f32, f32)> {
    let insights = StateInsights::of(state);
    let (vx, vy) = insights
        .new_hitters_of(actor)
        .into_iter()
        .next()
        .map(|(_, hit_vel)| hit_vel)?;
    let actor_trans = insights.transform_of(actor).unwrap();
    let rev_dir = notan::math::vec2(-*vx, -*vy).normalize();
    let target_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + rev_dir * 150.;
    Some((target_pos.x, target_pos.y))
}

pub(super) fn get_priority_actions(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    // Move towards the projectile.
    if let Some((target_x, target_y)) = try_move_towards_projectile(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
            x: target_x,
            y: target_y,
            scale_obstacles: true,
        })];
    }
    // Attack on sight.
    if let Some(target) = try_get_enemy_on_sight(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::Attack { target })];
    }
    return vec![];
}

pub(super) fn get_dpos(
    target_x: &f32,
    target_y: &f32,
    actor: &EntityRef,
    state: &State,
) -> Option<(f32, f32)> {
    let insights = StateInsights::of(state);
    let actor_pos = insights.transform_of(actor)?;
    Some((target_x - actor_pos.x, target_y - actor_pos.y))
}

pub(super) fn reached_destination(
    target_x: &f32,
    target_y: &f32,
    actor: &EntityRef,
    state: &State,
) -> bool {
    get_dpos(target_x, target_y, actor, state)
        .map(|dpos| dpos.0.abs() <= 5. && dpos.1.abs() <= 5.)
        .unwrap_or(true)
}
