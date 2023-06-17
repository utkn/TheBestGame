use crate::core::*;

/// A system that handles simple translation using the velocities.
#[derive(Clone, Copy, Debug, Default)]
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Position, Velocity)>()
            .for_each(|(e, (pos, vel))| {
                let mut new_pos = *pos;
                new_pos.x += vel.x * ctx.dt;
                new_pos.y += vel.y * ctx.dt;
                cmds.set_component(&e, new_pos);
            });
    }
}

/// A system that handles user control.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlSystem;

impl System for ControlSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(Velocity, TargetVelocity, Controller)>()
            .for_each(|(e, (_, _, controller))| {
                // Determine the target velocity according to the current pressed keys.
                let new_target_vel_x = if ctx.control_map.left_is_down {
                    -1.
                } else if ctx.control_map.right_is_down {
                    1.
                } else {
                    0.
                } * controller.max_speed;
                let new_target_vel_y = if ctx.control_map.up_is_down {
                    -1.
                } else if ctx.control_map.down_is_down {
                    1.
                } else {
                    0.
                } * controller.max_speed;
                // Set the target velocity.
                cmds.set_component(
                    &e,
                    TargetVelocity {
                        x: new_target_vel_x,
                        y: new_target_vel_y,
                    },
                )
            })
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
                let new_vel = vel + acc.0 * ctx.dt * (target_vel - vel).normalize_or_zero();
                cmds.set_component(
                    &e,
                    Velocity {
                        x: new_vel.x,
                        y: new_vel.y,
                    },
                )
            })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FaceMouseSystem;

impl System for FaceMouseSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        let (mouse_x, mouse_y) = ctx.control_map.mouse_pos;
        state
            .select::<(FaceMouse, Rotation, Position)>()
            .for_each(|(e, (_, _, pos))| {
                let mouse_pos = notan::math::vec2(mouse_x, mouse_y);
                let entity_pos = notan::math::vec2(pos.x, pos.y);
                let diff = mouse_pos - entity_pos;
                if diff.length_squared() == 0. {
                    return;
                }
                let new_rot = diff.angle_between(notan::math::vec2(1., 0.)).to_degrees();
                cmds.set_component(&e, Rotation { deg: new_rot });
            });
    }
}

/// A system that handles position and rotation anchoring.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorSystem;

impl System for AnchorSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(AnchorPosition,)>()
            .for_each(|(child_entity, (anchor,))| {
                if !state.is_valid(&anchor.0) {
                    // TODO: decide what to do
                    // cmds.remove_component::<AnchorPosition>(&child_entity);
                } else if let Some((anchored_pos,)) = state.select_one::<(Position,)>(&anchor.0) {
                    let new_pos = anchored_pos.translated(anchor.1);
                    cmds.set_component(&child_entity, new_pos);
                }
            });
        state
            .select::<(AnchorRotation,)>()
            .for_each(|(child_entity, (anchor,))| {
                if !state.is_valid(&anchor.0) {
                    // TODO: decide what to do
                    // cmds.remove_component::<AnchorRotation>(&child_entity);
                } else if let Some((anchored_rot,)) = state.select_one::<(Rotation,)>(&anchor.0) {
                    let new_rot = Rotation {
                        deg: anchored_rot.deg + anchor.1,
                    };
                    cmds.set_component(&child_entity, new_rot);
                }
            })
    }
}

/// A system that handles the entities with a lifetime.
#[derive(Clone, Copy, Debug, Default)]
pub struct LifetimeSystem;

impl System for LifetimeSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.select::<(Lifetime,)>().for_each(|(e, (lifetime,))| {
            // Remove the entities with ended lifetime.
            if lifetime.0 <= 0. {
                cmds.remove_entity(&e);
            } else {
                // Update the alive entities' lifetimes.
                let dt = ctx.dt;
                cmds.update_component(&e, move |lifetime: &mut Lifetime| {
                    lifetime.0 -= dt;
                });
            }
        });
    }
}
