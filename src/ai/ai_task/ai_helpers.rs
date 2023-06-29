use crate::{
    character::{CharacterBundle, CharacterInsights},
    physics::{Hitbox, ProjectileInsights},
    prelude::*,
    vehicle::VehicleInsights,
};

use super::{AiMovementHandler, AiTask, AiTaskOutput};

/// Returns the enemy on sight.
pub(super) fn try_get_enemy_on_sight<'a>(
    actor: &EntityRef,
    state: &'a impl StateReader,
) -> Option<EntityRef> {
    let ai_char = state.read_bundle::<CharacterBundle>(actor)?;
    let insights = StateInsights::of(state);
    let visibles = ai_char.visibles(state);
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

/// Returns the position to move to if the `actor` is hit by a projectile.
pub(super) fn try_move_towards_projectile(
    actor: &EntityRef,
    state: &impl StateReader,
) -> Option<(f32, f32)> {
    let insights = StateInsights::of(state);
    let (vx, vy) = insights
        .new_hitters_of(actor)
        .into_iter()
        .next()
        .map(|(_, hit_vel)| hit_vel)?;
    let ai_trans = insights.transform_of(actor)?;
    let rev_dir = notan::math::vec2(-*vx, -*vy).normalize();
    let target_pos = notan::math::vec2(ai_trans.x, ai_trans.y) + rev_dir * 40.;
    Some((target_pos.x, target_pos.y))
}

/// Returns the actions that have the priority.
pub(super) fn get_urgent_actions(actor: &EntityRef, state: &impl StateReader) -> Vec<AiTaskOutput> {
    // Move towards the projectile.
    if let Some(target_pos) = try_move_towards_projectile(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos(
            AiMovementHandler::new(target_pos),
        ))];
    }
    // Attack on sight.
    if let Some(target) = try_get_enemy_on_sight(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::Attack { target })];
    }
    return vec![];
}

/// Returns whether there are urgent actions that needs to be taken (should be handled by the `Routine` task).
pub(super) fn has_urgent_actions(actor: &EntityRef, state: &impl StateReader) -> bool {
    get_urgent_actions(actor, state).len() > 0
}

pub(super) fn get_dpos(
    target_x: &f32,
    target_y: &f32,
    actor: &EntityRef,
    state: &impl StateReader,
) -> Option<(f32, f32)> {
    let actor_trans = StateInsights::of(state).transform_of(actor)?;
    Some((target_x - actor_trans.x, target_y - actor_trans.y))
}

pub(super) fn reached_destination_approx(
    target_x: &f32,
    target_y: &f32,
    actor: &EntityRef,
    state: &impl StateReader,
) -> bool {
    let cell_size = {
        state
            .select_one::<(Hitbox,)>(actor)
            .map(|(hb,)| match hb.1 {
                crate::physics::Shape::Circle { r } => 2. * r,
                crate::physics::Shape::Rect { w, h } => w.max(h),
            })
            .unwrap_or(20.)
    };
    get_dpos(target_x, target_y, actor, state)
        .map(|dpos| dpos.0.abs() <= cell_size && dpos.1.abs() <= cell_size)
        .unwrap_or(true)
}
