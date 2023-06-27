use crate::{controller::ProximityInteractable, item::*, physics::*, prelude::*, sprite::Sprite};

use super::Vehicle;

#[derive(Clone, Copy, Debug)]
pub struct VehicleBundle {
    pub vehicle: EntityRef,
    pub door: EntityRef,
}

impl VehicleBundle {
    pub fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
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
            Hitbox(HitboxType::Dynamic, Shape::Rect { w: 100., h: 40. }),
            InteractTarget::<Hitbox>::default(),
            InteractTarget::<VisionField>::default(),
        ));
        cmds.set_components(
            &vehicle,
            (
                Name("basic car"),
                Sprite::new("basic_car", 3),
                ProximityInteractable,
                TargetRotation::default(),
                Equipment::new([EquipmentSlot::VehicleGas, EquipmentSlot::VehicleModule]),
                InteractTarget::<Equipment>::default(),
            ),
        );
        let door = cmds.create_from((
            Transform::default(),
            AnchorTransform(vehicle, (0., -40.), 0.),
            ProximityInteractable,
            UntargetedInteractionDelegate(vehicle),
            Hitbox(HitboxType::Ghost, Shape::Rect { w: 40., h: 30. }),
            InteractTarget::<Hitbox>::default(),
            ExistenceDependency(vehicle),
        ));
        cmds.push_bundle(Self { vehicle, door })
    }
}

impl<'a> EntityBundle<'a> for VehicleBundle {
    type TupleRepr = (EntityRef, EntityRef);

    fn primary_entity(&self) -> &EntityRef {
        &self.vehicle
    }

    fn deconstruct(self) -> (EntityRef, EntityRef) {
        (self.vehicle, self.door)
    }

    fn reconstruct(args: (&EntityRef, &EntityRef)) -> Self {
        Self {
            vehicle: *args.0,
            door: *args.1,
        }
    }
}
