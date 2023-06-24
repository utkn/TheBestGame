use crate::{
    ai::VisionField, controller::ProximityInteractable, item::*, physics::*, prelude::*,
    sprite::Sprite,
};

use super::Vehicle;

pub struct VehicleBundle {
    vehicle: EntityRef,
    door: EntityRef,
}

impl EntityBundle for VehicleBundle {
    fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
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
                Sprite::new("basic_car", 3),
                TargetRotation::default(),
                Equipment::new([EquipmentSlot::VehicleGas, EquipmentSlot::VehicleModule]),
                InteractTarget::<Equipment>::default(),
            ),
        );
        let door = cmds.create_from((
            Transform::default(),
            AnchorTransform(vehicle, (0., -20.)),
            ProximityInteractable,
            UntargetedInteractionDelegate(vehicle),
            Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
            InteractTarget::<Hitbox>::default(),
            ExistenceDependency(vehicle),
        ));
        Self { vehicle, door }
    }

    fn primary_entity(&self) -> &EntityRef {
        &self.vehicle
    }

    fn try_reconstruct(vehicle: &EntityRef, state: &State) -> Option<Self> {
        let door = state
            .select::<(
                ProximityInteractable,
                AnchorTransform,
                UntargetedInteractionDelegate,
            )>()
            .find(|(_, (_, anchor, intr_delegate))| {
                &anchor.0 == vehicle && &intr_delegate.0 == vehicle
            })
            .map(|(e, _)| e)?;
        Some(Self {
            vehicle: *vehicle,
            door,
        })
    }
}
