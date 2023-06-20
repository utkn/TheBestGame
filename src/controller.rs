use std::marker::PhantomData;

use crate::{
    core::*,
    interaction::{EndProximityInteractReq, StartProximityInteractReq},
};

/// Entities with this component will be able to be moved by the given [`ControlDriver`].
#[derive(Clone, Debug)]
pub struct Controller<D: ControlDriver>(pub D);

pub enum ControlCommand {
    SetTargetVelocity(TargetVelocity),
    ProximityInteract,
    ProximityUninteract,
    None,
}

pub trait ControlDriver: 'static + Clone + std::fmt::Debug {
    fn get_command(&self, ctx: &UpdateContext, state: &State, actor: &EntityRef) -> ControlCommand;
}

/// A [`ControlDriver`] using key presses to emit [`ControlCommand`]s.
#[derive(Clone, Copy, Debug)]
pub struct UserInput {
    pub default_speed: f32,
}

impl ControlDriver for UserInput {
    fn get_command(&self, ctx: &UpdateContext, state: &State, actor: &EntityRef) -> ControlCommand {
        // Try to start the proximity interaction with explicit key press.
        if ctx.control_map.start_interact_was_pressed {
            return ControlCommand::ProximityInteract;
        }
        // Try to end a proximity interaction with explicit key press.
        if ctx.control_map.end_interact_was_pressed {
            return ControlCommand::ProximityUninteract;
        }
        let speed = state
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
        ControlCommand::SetTargetVelocity(TargetVelocity {
            x: new_target_vel_x,
            y: new_target_vel_y,
        })
    }
}

/// A system that handles user control.
#[derive(Clone, Copy, Debug)]
pub struct ControlSystem<A: ControlDriver>(PhantomData<A>);

impl<D: ControlDriver> Default for ControlSystem<D> {
    fn default() -> Self {
        Self(Default::default())
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
                match controller.0.get_command(ctx, state, &actor) {
                    ControlCommand::SetTargetVelocity(vel) => {
                        cmds.set_component::<TargetVelocity>(&actor, vel)
                    }
                    ControlCommand::ProximityInteract => {
                        cmds.emit_event(StartProximityInteractReq(actor))
                    }
                    ControlCommand::ProximityUninteract => {
                        cmds.emit_event(EndProximityInteractReq(actor))
                    }
                    ControlCommand::None => {}
                }
            });
    }
}
