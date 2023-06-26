use crate::prelude::*;

/// A system that handles simple translation using the velocities.
#[derive(Clone, Copy, Debug, Default)]
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Transform, Velocity)>()
            .for_each(|(e, (trans, vel))| {
                // No movement for anchored entities!
                if let Some(_) = state.select_one::<(AnchorTransform,)>(&e) {
                    return;
                };
                let (mut new_pos_x, mut new_pos_y) = (trans.x, trans.y);
                new_pos_x += vel.x * ctx.dt;
                new_pos_y += vel.y * ctx.dt;
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

impl System for ApproachVelocitySystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Velocity, TargetVelocity, Acceleration)>()
            .for_each(|(e, (vel, target_vel, acc))| {
                let vel = notan::math::vec2(vel.x, vel.y);
                let target_vel = notan::math::vec2(target_vel.x, target_vel.y);
                if vel.distance_squared(target_vel) < 1. {
                    cmds.set_component(
                        &e,
                        Velocity {
                            x: target_vel.x,
                            y: target_vel.y,
                        },
                    );
                } else {
                    let new_vel = vel + acc.0 * ctx.dt * (target_vel - vel).normalize_or_zero();
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

impl System for ApproachRotationSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Transform, TargetRotation, Acceleration)>()
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

impl System for AnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(AnchorTransform, Transform)>()
            .for_each(|(child_entity, (anchor, _))| {
                if !state.is_valid(&anchor.0) {
                    cmds.remove_component::<AnchorTransform>(&child_entity);
                } else if let Some((parent_trans,)) = state.select_one::<(Transform,)>(&anchor.0) {
                    // Translate by the offset.
                    let new_trans = parent_trans.translated(anchor.1);
                    cmds.set_component(&child_entity, new_trans);
                }
            });
    }
}

/// A system that handles position and rotation anchoring.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExistenceDependencySystem;

impl System for ExistenceDependencySystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(ExistenceDependency,)>()
            .for_each(|(child_entity, (dependency,))| {
                if !state.is_valid(&dependency.0) {
                    cmds.mark_for_removal(&child_entity);
                }
            });
    }
}

/// A system that handles the entities with a lifetime.
#[derive(Clone, Copy, Debug, Default)]
pub struct LifetimeSystem;

impl System for LifetimeSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
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
