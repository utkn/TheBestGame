use std::collections::HashSet;

use crate::core::*;
use crate::interaction::InteractionType;
use crate::physics::{CollisionState, Hitbox, HitboxType};

#[derive(Clone, Copy, Debug)]
pub struct Ridable;

#[derive(Clone, Debug)]
pub struct RideInteraction;

impl InteractionType for RideInteraction {
    fn valid_actors(target: &EntityRef, state: &State) -> Option<HashSet<EntityRef>> {
        let (_, coll_state) = state.select_one::<(Ridable, CollisionState)>(target)?;
        Some(coll_state.colliding.iter().cloned().collect())
    }

    fn on_start(actor: &EntityRef, target: &EntityRef, state: &State, cmds: &mut StateCommands) {
        let driver = actor;
        let vehicle = target;
        // Transfer the controller.
        if let Some((driver_controller,)) = state.select_one::<(Controller,)>(driver) {
            cmds.remove_component::<Controller>(driver);
            cmds.set_component(vehicle, driver_controller.clone());
        }
        // Anchor the driver to the vehicle.
        cmds.set_component(driver, AnchorTransform(*vehicle, (0., 0.)));
        if let Some((vehicle_transform,)) = state.select_one::<(Transform,)>(vehicle) {
            cmds.set_component(driver, vehicle_transform.clone());
        }
        // Give vehicle a dynamic hitbox.
        cmds.update_component(vehicle, |hb: &mut Hitbox| {
            hb.0 = HitboxType::Dynamic;
        });
    }

    fn on_end(actor: &EntityRef, target: &EntityRef, state: &State, cmds: &mut StateCommands) {
        let driver = actor;
        let vehicle = target;
        // Transfer the controller.
        if let Some((vehicle_controller,)) = state.select_one::<(Controller,)>(vehicle) {
            cmds.remove_component::<Controller>(vehicle);
            cmds.set_component(driver, vehicle_controller.clone());
        }
        // Stop moving.
        cmds.set_component(driver, TargetVelocity { x: 0., y: 0. });
        cmds.set_component(vehicle, TargetVelocity { x: 0., y: 0. });
        // Remove the driver's anchor.
        cmds.remove_component::<AnchorTransform>(driver);
        // Give vehicle a ghost hitbox.
        cmds.update_component(vehicle, |hb: &mut Hitbox| {
            hb.0 = HitboxType::Ghost;
        });
    }
}
