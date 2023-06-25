use std::marker::PhantomData;

use crate::{item::EquipmentSlot, prelude::*};

mod equipment_interaction;
mod proximity_interaction;
mod user_input_driver;

use equipment_interaction::*;
use proximity_interaction::*;

pub use equipment_interaction::HandInteractionSystem;
pub use proximity_interaction::{ProximityInteractable, ProximityInteractionSystem};
pub use user_input_driver::*;

/// Entities with this component will be able to be moved by the given [`ControlDriver`].
#[derive(Clone, Debug)]
pub struct Controller<D: ControlDriver>(pub D);

/// A command that can be emitted by a [`ControlDriver`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlCommand {
    SetTargetRotation(f32),
    SetTargetVelocity(f32, f32),
    ProximityInteract,
    ProximityUninteract,
    EquipmentInteract(EquipmentSlot),
    EquipmentUninteract(EquipmentSlot),
}

pub trait ControlDriver: 'static + Clone + std::fmt::Debug {
    /// Returns [`ControlCommand`]s from the given state of the game.
    fn get_commands(
        &mut self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
    ) -> Vec<ControlCommand>;
}

/// A system that handles user control.
#[derive(Clone, Debug)]
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
                let mut updated_driver = controller.0.clone();
                let controller_cmds = updated_driver.get_commands(&actor, ctx, state);
                cmds.set_component(&actor, Controller(updated_driver));
                controller_cmds.into_iter().for_each(|cmd| match cmd {
                    ControlCommand::SetTargetVelocity(vx, vy) => cmds
                        .set_component::<TargetVelocity>(&actor, TargetVelocity { x: vx, y: vy }),
                    ControlCommand::SetTargetRotation(deg) => {
                        cmds.update_component(&actor, move |target_rot: &mut TargetRotation| {
                            target_rot.deg = deg
                        });
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
