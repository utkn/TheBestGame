use crate::{
    controller::{CopyControllersReq, DeleteControllersReq},
    physics::{Hitbox, HitboxType},
    prelude::*,
};

use super::{Vehicle, VehicleBundle};

#[derive(Clone, Default, Debug)]
pub struct VehicleSystem;

impl<R: StateReader> System<R> for VehicleSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        state
            .read_events::<InteractionStartedEvt<Vehicle>>()
            .for_each(|evt| {
                let driver = &evt.actor;
                let vehicle = &evt.target;
                // Copy the driver's controllers to the vehicle.
                cmds.emit_event(CopyControllersReq::new(*driver, *vehicle));
                // Anchor the driver to the vehicle.
                cmds.set_component(driver, AnchorTransform(*vehicle, (0., 0.), 0.));
                if let Some((vehicle_transform,)) = state.select_one::<(Transform,)>(vehicle) {
                    cmds.set_component(driver, vehicle_transform.clone());
                }
                // Update driver's hitbox to ghost.
                cmds.update_component(driver, |hb: &mut Hitbox| {
                    hb.0 = HitboxType::Ghost;
                });
            });
        state
            .read_events::<InteractionEndedEvt<Vehicle>>()
            .for_each(|evt| {
                let driver = &evt.actor;
                let vehicle = &evt.target;
                // Remove the controllers from the vehicle.
                cmds.emit_event(DeleteControllersReq(*vehicle));
                // Stop moving.
                cmds.set_component(driver, TargetVelocity { x: 0., y: 0. });
                cmds.set_component(vehicle, TargetVelocity { x: 0., y: 0. });
                // Remove the driver's anchor.
                cmds.remove_component::<AnchorTransform>(driver);
                // Update driver's hitbox to dynamic.
                cmds.update_component(driver, |hb: &mut Hitbox| {
                    hb.0 = HitboxType::Dynamic;
                });
                // Position the driver at the door.
                // TODO: check if it is occupied before trying to do so.
                if let Some(vehicle_bundle) = state.read_bundle::<VehicleBundle>(vehicle) {
                    let door_transform = *StateInsights::of(state)
                        .transform_of(&vehicle_bundle.door)
                        .unwrap();
                    cmds.update_component(driver, move |trans: &mut Transform| {
                        trans.x = door_transform.x;
                        trans.y = door_transform.y;
                    });
                }
            });
    }
}
