use crate::vehicle::VehicleInsights;

use super::*;

/// Uses key presses to control the entities.
#[derive(Clone, Copy, Debug)]
pub struct UserInputDriver;

impl ControlDriver for UserInputDriver {
    fn get_commands(
        &mut self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
    ) -> Vec<ControlCommand> {
        if StateInsights::of(game_state).is_vehicle(actor) {
            get_vehicle_commands(actor, ctx, game_state, 200.)
        } else {
            get_character_commands(actor, ctx, game_state, 200.)
        }
    }
}

fn get_character_commands(
    actor: &EntityRef,
    ctx: &UpdateContext,
    game_state: &State,
    default_speed: f32,
) -> Vec<ControlCommand> {
    let mut vel_cmds = {
        // Try to start the proximity interaction with explicit key press.
        if ctx.control_map.start_interact_was_pressed {
            return vec![ControlCommand::ProximityInteract];
        }
        // Try to end a proximity interaction with explicit key press.
        if ctx.control_map.end_interact_was_pressed {
            return vec![ControlCommand::ProximityUninteract];
        }
        if ctx.control_map.mouse_left_was_pressed {
            return vec![ControlCommand::EquipmentInteract(EquipmentSlot::LeftHand)];
        }
        if ctx.control_map.mouse_right_was_pressed {
            return vec![ControlCommand::EquipmentInteract(EquipmentSlot::RightHand)];
        }
        if ctx.control_map.mouse_left_was_released {
            return vec![ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand)];
        }
        if ctx.control_map.mouse_right_was_released {
            return vec![ControlCommand::EquipmentUninteract(
                EquipmentSlot::RightHand,
            )];
        }
        let speed = game_state
            .select_one::<(MaxSpeed,)>(&actor)
            .map(|(max_speed,)| max_speed.0)
            .unwrap_or(default_speed);
        // Determine the target velocity according to the current pressed keys.
        let new_target_vel_x = if ctx.control_map.left_is_down {
            -1.
        } else if ctx.control_map.right_is_down {
            1.
        } else {
            0.
        } * speed;
        let new_target_vel_y = if ctx.control_map.up_is_down {
            -1.
        } else if ctx.control_map.down_is_down {
            1.
        } else {
            0.
        } * speed;
        // Set the target velocity.
        vec![ControlCommand::SetTargetVelocity(
            new_target_vel_x,
            new_target_vel_y,
        )]
    };
    let trans = game_state
        .select_one::<(Transform,)>(actor)
        .map(|(trans,)| *trans)
        .unwrap_or_default();
    let (mouse_x, mouse_y) = ctx.control_map.mouse_pos;
    let mouse_pos = notan::math::vec2(mouse_x, mouse_y);
    let entity_pos = notan::math::vec2(trans.x, trans.y);
    let diff = mouse_pos - entity_pos;
    if diff.length_squared() > 0. {
        let new_deg = diff.angle_between(notan::math::vec2(1., 0.)).to_degrees();
        vel_cmds.push(ControlCommand::SetTargetRotation(new_deg));
    }
    vel_cmds
}
fn get_vehicle_commands(
    actor: &EntityRef,
    ctx: &UpdateContext,
    game_state: &State,
    default_speed: f32,
) -> Vec<ControlCommand> {
    let (curr_trans, _curr_vel) = game_state
        .select_one::<(Transform, Velocity)>(actor)
        .map(|(trans, vel)| (*trans, *vel))
        .unwrap_or_default();
    let speed = game_state
        .select_one::<(MaxSpeed,)>(&actor)
        .map(|(max_speed,)| max_speed.0)
        .unwrap_or(default_speed);
    let directional_speed = if ctx.control_map.up_is_down {
        1.
    } else if ctx.control_map.down_is_down {
        -0.5
    } else {
        0.
    } * speed;
    let (dir_vec_x, dir_vec_y) = curr_trans.dir_vec();
    let dir_vec = notan::math::vec2(dir_vec_x, dir_vec_y);
    let new_target_vel = dir_vec * directional_speed;
    let mut delta_deg = if ctx.control_map.left_is_down {
        1.
    } else if ctx.control_map.right_is_down {
        -1.
    } else {
        0.
    } * 20.;
    if ctx.control_map.down_is_down {
        delta_deg *= -1.;
    } else if ctx.control_map.up_is_down {
        delta_deg *= 1.;
    } else {
        delta_deg *= 0.;
    }
    vec![
        ControlCommand::SetTargetVelocity(new_target_vel.x, new_target_vel.y),
        ControlCommand::SetTargetRotation(curr_trans.deg + delta_deg),
    ]
}
