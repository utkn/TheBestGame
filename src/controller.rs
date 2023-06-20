use std::{collections::HashMap, marker::PhantomData};

use crate::{
    interaction::{
        EndProximityInteractReq, HandInteractReq, HandSide, HandUninteractReq,
        StartProximityInteractReq,
    },
    prelude::*,
};

/// Entities with this component will be able to be moved by the given [`ControlDriver`].
#[derive(Clone, Debug)]
pub struct Controller<D: ControlDriver>(pub D);

/// A command that can be emitted by a [`ControlDriver`].
pub enum ControlCommand {
    SetTargetVelocity(TargetVelocity),
    ProximityInteract,
    ProximityUninteract,
    HandInteract(HandSide),
    HandUninteract(HandSide),
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
pub struct UserInputDriver {
    pub default_speed: f32,
}

impl ControlDriver for UserInputDriver {
    type State = ();
    fn get_commands(
        &self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
        _: &mut Self::State,
    ) -> Vec<ControlCommand> {
        // Try to start the proximity interaction with explicit key press.
        if ctx.control_map.start_interact_was_pressed {
            return vec![ControlCommand::ProximityInteract];
        }
        // Try to end a proximity interaction with explicit key press.
        if ctx.control_map.end_interact_was_pressed {
            return vec![ControlCommand::ProximityUninteract];
        }
        if ctx.control_map.mouse_left_was_pressed {
            return vec![ControlCommand::HandInteract(HandSide::Left)];
        }
        if ctx.control_map.mouse_right_was_pressed {
            return vec![ControlCommand::HandInteract(HandSide::Right)];
        }
        if ctx.control_map.mouse_left_was_released {
            return vec![ControlCommand::HandUninteract(HandSide::Left)];
        }
        if ctx.control_map.mouse_right_was_released {
            return vec![ControlCommand::HandUninteract(HandSide::Left)];
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
                        ControlCommand::ProximityInteract => {
                            cmds.emit_event(StartProximityInteractReq(actor))
                        }
                        ControlCommand::ProximityUninteract => {
                            cmds.emit_event(EndProximityInteractReq(actor))
                        }
                        ControlCommand::HandInteract(hand) => {
                            cmds.emit_event(HandInteractReq(actor, hand))
                        }
                        ControlCommand::HandUninteract(hand) => {
                            cmds.emit_event(HandUninteractReq(actor, hand))
                        }
                    });
            });
    }
}
