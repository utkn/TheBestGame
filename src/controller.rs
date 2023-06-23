use std::{collections::HashMap, marker::PhantomData};

use crate::{item::EquipmentSlot, prelude::*};

mod equipment_interaction;
mod proximity_interaction;

use equipment_interaction::*;
use proximity_interaction::*;

pub use equipment_interaction::HandInteractionSystem;
pub use proximity_interaction::{ProximityInteractable, ProximityInteractionSystem};

/// Entities with this component will be able to be moved by the given [`ControlDriver`].
#[derive(Clone, Debug)]
pub struct Controller<D: ControlDriver>(pub D);

/// A command that can be emitted by a [`ControlDriver`].
pub enum ControlCommand {
    SetTargetRotation(f32),
    SetTargetVelocity(TargetVelocity),
    ProximityInteract,
    ProximityUninteract,
    EquipmentInteract(EquipmentSlot),
    EquipmentUninteract(EquipmentSlot),
}

pub trait ControlDriver: 'static + Clone + std::fmt::Debug {
    /// The type of the mutable state of the driver.
    type State: Default;
    /// Returns [`ControlCommand`]s from the given state of itself and the game.
    fn get_commands(
        &self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
        driver_state: &mut Self::State,
    ) -> Vec<ControlCommand>;
}

/// Uses key presses to control the entities.
#[derive(Clone, Copy, Debug)]
pub struct UserInputCharacterDriver {
    pub default_speed: f32,
}

impl ControlDriver for UserInputCharacterDriver {
    type State = ();
    fn get_commands(
        &self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
        _: &mut Self::State,
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
                .unwrap_or(self.default_speed);
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
            vec![ControlCommand::SetTargetVelocity(TargetVelocity {
                x: new_target_vel_x,
                y: new_target_vel_y,
            })]
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
}

/// Uses key presses to control the entities.
#[derive(Clone, Copy, Debug)]
pub struct UserInputVehicleDriver;

impl ControlDriver for UserInputVehicleDriver {
    type State = ();

    fn get_commands(
        &self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
        driver_state: &mut Self::State,
    ) -> Vec<ControlCommand> {
        let curr_trans = game_state
            .select_one::<(Transform,)>(actor)
            .map(|(trans,)| *trans)
            .unwrap_or_default();
        let delta_deg = if ctx.control_map.left_is_down {
            1.
        } else if ctx.control_map.right_is_down {
            -1.
        } else {
            0.
        } * 50.;
        let speed = game_state
            .select_one::<(MaxSpeed,)>(&actor)
            .map(|(max_speed,)| max_speed.0)
            .unwrap_or(20.);
        let directional_speed = if ctx.control_map.up_is_down {
            1.
        } else if ctx.control_map.down_is_down {
            -1.
        } else {
            0.
        } * speed;
        let new_target_vel =
            notan::math::Vec2::from_angle(-curr_trans.deg.to_radians()) * directional_speed;
        let new_target_vel = TargetVelocity {
            x: new_target_vel.x,
            y: new_target_vel.y,
        };
        vec![
            ControlCommand::SetTargetVelocity(new_target_vel),
            ControlCommand::SetTargetRotation(curr_trans.deg + delta_deg),
        ]
    }
}

/// A system that handles user control.
#[derive(Clone, Debug)]
pub struct ControlSystem<A: ControlDriver> {
    states: HashMap<EntityRef, A::State>,
    pd: PhantomData<A>,
}

impl<D: ControlDriver> Default for ControlSystem<D> {
    fn default() -> Self {
        Self {
            states: Default::default(),
            pd: Default::default(),
        }
    }
}

/// Requests the [`Controller<_>`]s of `from` entity to be copied into the `to` entity.
#[derive(Clone, Copy, Debug)]
pub struct CopyControllersReq {
    from: EntityRef,
    to: EntityRef,
}
impl CopyControllersReq {
    pub fn new(from: EntityRef, to: EntityRef) -> Self {
        Self { from, to }
    }
}

/// Requests the [`Controller<_>`]s of the given entity to be removed.
#[derive(Clone, Copy, Debug)]
pub struct DeleteControllersReq(pub EntityRef);

impl<D: ControlDriver> System for ControlSystem<D> {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<CopyControllersReq>().for_each(|evt| {
            if let Some((from_controller,)) = state.select_one::<(Controller<D>,)>(&evt.from) {
                cmds.set_component(&evt.to, from_controller.clone());
            }
        });
        state.read_events::<DeleteControllersReq>().for_each(|evt| {
            cmds.remove_component::<Controller<D>>(&evt.0);
        });
        state
            .select::<(Controller<D>,)>()
            .for_each(|(actor, (controller,))| {
                let driver_state = self.states.entry(actor).or_default();
                controller
                    .0
                    .get_commands(&actor, ctx, state, driver_state)
                    .into_iter()
                    .for_each(|cmd| match cmd {
                        ControlCommand::SetTargetVelocity(vel) => {
                            cmds.set_component::<TargetVelocity>(&actor, vel)
                        }
                        ControlCommand::SetTargetRotation(deg) => {
                            cmds.update_component(
                                &actor,
                                move |target_rot: &mut TargetRotation| target_rot.deg = deg,
                            );
                        }
                        ControlCommand::ProximityInteract => {
                            cmds.emit_event(StartProximityInteractReq(actor))
                        }
                        ControlCommand::ProximityUninteract => {
                            cmds.emit_event(EndProximityInteractReq(actor))
                        }
                        ControlCommand::EquipmentInteract(slot) => {
                            cmds.emit_event(EquipmentInteractReq(actor, slot))
                        }
                        ControlCommand::EquipmentUninteract(slot) => {
                            cmds.emit_event(EquipmentUninteractReq(actor, slot))
                        }
                    });
            });
    }
}
