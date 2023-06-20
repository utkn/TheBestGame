use crate::core::*;

use crate::interaction::{Interaction, InteractionEndedEvt, InteractionStartedEvt};
use crate::storage::Storage;

#[derive(Clone, Copy, Debug)]
pub struct Vehicle;

impl Interaction for Vehicle {
    fn priority() -> usize {
        Storage::priority() + 10
    }

    fn can_start(_: &EntityRef, target: &EntityRef, state: &State) -> bool {
        state.select_one::<(Vehicle,)>(target).is_some()
    }
}

#[derive(Clone, Default, Debug)]
pub struct VehicleSystem;

impl System for VehicleSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .read_events::<InteractionStartedEvt<Vehicle>>()
            .for_each(|evt| {
                let driver = &evt.actor;
                let vehicle = &evt.target;
                // Copy the controller to the vehicle.
                if let Some((driver_controller,)) = state.select_one::<(Controller,)>(driver) {
                    cmds.set_component(vehicle, driver_controller.clone());
                }
                // Anchor the driver to the vehicle.
                cmds.set_component(driver, AnchorTransform(*vehicle, (0., 0.)));
                if let Some((vehicle_transform,)) = state.select_one::<(Transform,)>(vehicle) {
                    cmds.set_component(driver, vehicle_transform.clone());
                }
            });
        state
            .read_events::<InteractionEndedEvt<Vehicle>>()
            .for_each(|evt| {
                let driver = &evt.actor;
                let vehicle = &evt.target;
                // Transfer back the controller.
                cmds.remove_component::<Controller>(vehicle);
                // Stop moving.
                cmds.set_component(driver, TargetVelocity { x: 0., y: 0. });
                cmds.set_component(vehicle, TargetVelocity { x: 0., y: 0. });
                // Remove the driver's anchor.
                cmds.remove_component::<AnchorTransform>(driver);
                // Position the driver a bit right for collision detection to resolve it.
                cmds.update_component(driver, |trans: &mut Transform| {
                    trans.x += 10.;
                });
            });
    }
}
