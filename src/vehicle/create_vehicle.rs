use crate::{
    ai::VisionField, controller::ProximityInteractable, item::*, physics::*, prelude::*,
    sprite::Sprite,
};

use super::Vehicle;

pub fn create_vehicle(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    let vehicle = cmds.create_from((
        trans,
        Vehicle,
        InteractTarget::<Vehicle>::default(),
        Velocity::default(),
        TargetVelocity::default(),
        Acceleration(2000.),
        MaxSpeed(1000.),
        Storage::new(6),
        InteractTarget::<Storage>::default(),
        Hitbox(HitboxType::Dynamic, Shape::Rect(100., 40.)),
        InteractTarget::<Hitbox>::default(),
        InteractTarget::<VisionField>::default(),
    ));
    cmds.set_components(
        &vehicle,
        (
            Name("basic car"),
            Sprite("basic_car"),
            TargetRotation::default(),
            Equipment::new([EquipmentSlot::VehicleGas, EquipmentSlot::VehicleModule]),
            InteractTarget::<Equipment>::default(),
        ),
    );
    let _vehicle_door = cmds.create_from((
        Transform::default(),
        AnchorTransform(vehicle, (0., -20.)),
        ProximityInteractable,
        UntargetedInteractionDelegate(vehicle),
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
        InteractTarget::<Hitbox>::default(),
        ExistenceDependency(vehicle),
    ));
    vehicle
}
