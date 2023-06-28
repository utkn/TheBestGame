use notan::math;

use crate::prelude::*;

/// A system that handles simple translation using the velocities.
#[derive(Clone, Copy, Debug, Default)]
pub struct MovementSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for MovementSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state
            .select::<(Transform, Velocity)>()
            // No movement for anchored entities!
            .filter(|(e, _)| state.select_one::<(AnchorTransform,)>(e).is_none())
            .for_each(|(e, (trans, vel))| {
                let (mut new_pos_x, mut new_pos_y) = (trans.x, trans.y);
                let (dx, dy) = (vel.x * ctx.dt, vel.y * ctx.dt);
                new_pos_x += dx;
                new_pos_y += dy;
                cmds.update_component(&e, move |trans: &mut Transform| {
                    trans.x = new_pos_x;
                    trans.y = new_pos_y;
                });
            });
    }
}

/// A system that handles accelerating to a target velocity.
#[derive(Clone, Copy, Debug, Default)]
pub struct ApproachVelocitySystem;

impl<R: StateReader, W: StateWriter> System<R, W> for ApproachVelocitySystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state
            .select::<(Velocity, TargetVelocity, Acceleration)>()
            // No movement for anchored entities!
            .filter(|(e, _)| state.select_one::<(AnchorTransform,)>(e).is_none())
            .for_each(|(e, (vel, target_vel, acc))| {
                let vel = notan::math::vec2(vel.x, vel.y);
                let target_vel = notan::math::vec2(target_vel.x, target_vel.y);
                // who cares ??
                if vel.distance_squared(target_vel) <= 1. {
                    return;
                } else {
                    let dv = acc.0 * ctx.dt * (target_vel - vel).normalize_or_zero();
                    let mut new_vel = vel + dv;
                    // Clamp to the target velocity.
                    new_vel.x = if (dv.x > 0. && new_vel.x >= target_vel.x)
                        || (dv.x < 0. && new_vel.x <= target_vel.x)
                    {
                        target_vel.x
                    } else {
                        new_vel.x
                    };
                    new_vel.y = if (dv.y > 0. && new_vel.y >= target_vel.y)
                        || (dv.y < 0. && new_vel.y <= target_vel.y)
                    {
                        target_vel.y
                    } else {
                        new_vel.y
                    };
                    cmds.set_component(
                        &e,
                        Velocity {
                            x: new_vel.x,
                            y: new_vel.y,
                        },
                    )
                }
            })
    }
}

/// A system that handles rotating to a target rotation.
#[derive(Clone, Copy, Debug, Default)]
pub struct ApproachRotationSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for ApproachRotationSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state
            .select::<(Transform, TargetRotation, Acceleration)>()
            // No movement for anchored entities!
            .filter(|(e, _)| state.select_one::<(AnchorTransform,)>(e).is_none())
            .for_each(|(e, (trans, target_rot, acc))| {
                let curr_deg = trans.deg;
                let target_deg = target_rot.deg;
                let mut diff = target_deg - curr_deg;
                // Make sure we take the shortest path.
                if diff > 270. {
                    diff -= 360.;
                }
                if diff < -270. {
                    diff += 360.;
                }
                // Approach to the target rotation with a cool harding effect.
                let new_rot = curr_deg + diff.signum() * ctx.dt * diff.abs() * acc.0 / 400.;
                cmds.update_component(&e, move |trans: &mut Transform| {
                    trans.deg = new_rot;
                });
            })
    }
}

/// A system that handles position and rotation anchoring.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for AnchorSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state
            .select::<(AnchorTransform, Transform)>()
            .for_each(|(child_entity, (anchor, _))| {
                if !state.is_valid(&anchor.0) {
                    cmds.remove_component::<AnchorTransform>(&child_entity);
                } else if let Some((parent_trans,)) = state.select_one::<(Transform,)>(&anchor.0) {
                    let offset = anchor.1;
                    let rotated_offset = math::Vec2::from_angle(-parent_trans.deg.to_radians())
                        .rotate(math::vec2(offset.0, offset.1));
                    // Translate by the offset.
                    let new_trans = parent_trans
                        .translated((rotated_offset.x, rotated_offset.y))
                        .with_deg(parent_trans.deg + anchor.2);
                    cmds.set_component(&child_entity, new_trans);
                }
            });
    }
}

/// A system that handles the entities with a lifetime.
#[derive(Clone, Copy, Debug, Default)]
pub struct LifetimeSystem;

impl<R: StateReader, W: StateWriter> System<R, W> for LifetimeSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &R, cmds: &mut W) {
        state.select::<(Lifetime,)>().for_each(|(e, (lifetime,))| {
            // Remove the entities with ended lifetime.
            if lifetime.remaining_time <= 0. {
                cmds.mark_for_removal(&e);
            } else {
                // Update the alive entities' lifetimes.
                let dt = ctx.dt;
                cmds.update_component(&e, move |lifetime: &mut Lifetime| {
                    lifetime.remaining_time -= dt;
                });
            }
        });
    }
}
