use crate::prelude::*;

use super::Vehicle;

pub trait VehicleInsights<'a> {
    /// Returns true iff the given entity is a vehicle.
    fn is_vehicle(&self, e: &EntityRef) -> bool;
    /// Returns the driver of the given `vehicle_entity`.
    fn driver_of(&self, vehicle_entity: &EntityRef) -> Option<&'a EntityRef>;
    /// Returns the vehicle entity that `actor` is driving.
    fn vehicle_of(&self, actor: &EntityRef) -> Option<EntityRef>;
}

impl<'a, R: StateReader> VehicleInsights<'a> for StateInsights<'a, R> {
    fn is_vehicle(&self, e: &EntityRef) -> bool {
        self.0.select_one::<(Vehicle,)>(e).is_some()
    }

    fn driver_of(&self, vehicle_entity: &EntityRef) -> Option<&'a EntityRef> {
        self.0
            .select_one::<(Vehicle, InteractTarget<Vehicle>)>(vehicle_entity)
            .and_then(|(_, vehicle_intr)| vehicle_intr.actors.iter().next())
    }

    fn vehicle_of(&self, actor: &EntityRef) -> Option<EntityRef> {
        self.0
            .select::<(InteractTarget<Vehicle>,)>()
            .find(|(_, (vehicle_intr,))| vehicle_intr.actors.contains(actor))
            .map(|(e, _)| e)
    }
}
