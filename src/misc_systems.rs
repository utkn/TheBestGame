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
                let new_target_vel_x = if ctx.control_map.left_pressed {
                    -1.
                } else if ctx.control_map.right_pressed {
                    1.
                } else {
                    0.
                } * controller.max_speed;
                let new_target_vel_y = if ctx.control_map.up_pressed {
                    -1.
                } else if ctx.control_map.down_pressed {
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

/// A system that handles position anchoring.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorPositionSystem;

impl System for AnchorPositionSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(AnchorPosition,)>()
            .for_each(|(child_entity, (anchor,))| {
                if !state.is_valid(&anchor.0) {
                    cmds.remove_entity(&child_entity);
                } else if let Some((anchored_pos,)) = state.select_one::<(Position,)>(&anchor.0) {
                    let new_pos = anchored_pos.translated(anchor.1);
                    cmds.set_component(&child_entity, new_pos);
                }
            })
    }
}
