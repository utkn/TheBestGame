use crate::{item::*, needs::*, physics::*, prelude::*};

use super::Character;

#[derive(Clone, Copy, Debug)]
pub struct CharacterBundle {
    pub character: EntityRef,
    pub vision_field: EntityRef,
}

impl CharacterBundle {
    pub fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
        let character = cmds.create_from((
            trans,
            Character,
            Velocity::default(),
            Acceleration(2000.),
            TargetVelocity::default(),
            MaxSpeed(300.),
            Hitbox(HitboxType::Dynamic, Shape::Rect { w: 20., h: 20. }),
            InteractTarget::<Hitbox>::default(),
            Storage::new(15),
            Equipment::new([
                EquipmentSlot::Head,
                EquipmentSlot::Torso,
                EquipmentSlot::Backpack,
                EquipmentSlot::LeftHand,
                EquipmentSlot::RightHand,
                EquipmentSlot::Legs,
                EquipmentSlot::Feet,
            ]),
            Needs::new([
                (NeedType::Health, NeedStatus::with_max(100.)),
                (NeedType::Energy, NeedStatus::with_max(100.)),
                (NeedType::Sanity, NeedStatus::with_max(100.)),
                (NeedType::Hunger, NeedStatus::with_zero(100.)),
                (NeedType::Thirst, NeedStatus::with_zero(100.)),
            ]),
            InteractTarget::<VisionField>::default(),
        ));
        cmds.set_components(&character, (TargetRotation::default(),));
        let vision_field = cmds.create_from((
            Transform::default(),
            AnchorTransform(character, (200., 0.)),
            Hitbox(HitboxType::Ghost, Shape::Circle { r: 200. }),
            InteractTarget::<Hitbox>::default(),
            VisionField(200.),
        ));
        cmds.push_bundle(Self {
            character,
            vision_field,
        })
    }
}

impl<'a> EntityBundle<'a> for CharacterBundle {
    type TupleRepr = (EntityRef, EntityRef);

    fn primary_entity(&self) -> &EntityRef {
        &self.character
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (self.character, self.vision_field)
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            character: *args.0,
            vision_field: *args.1,
        }
    }
}
