use crate::{ai::VisionField, item::*, needs::*, physics::*, prelude::*};

use super::Character;

pub struct CharacterBundle {
    pub character: EntityRef,
    pub vision_field: EntityRef,
}

impl EntityBundle for CharacterBundle {
    fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
        let character = cmds.create_from((
            trans,
            Character,
            Velocity::default(),
            Acceleration(2000.),
            TargetVelocity::default(),
            MaxSpeed(300.),
            Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
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
            AnchorTransform(character, (0., 0.)),
            Hitbox(HitboxType::Ghost, Shape::Circle(50.)),
            InteractTarget::<Hitbox>::default(),
            VisionField(50.),
        ));
        Self {
            character,
            vision_field,
        }
    }

    fn primary_entity(&self) -> &EntityRef {
        &self.character
    }

    fn try_reconstruct(character: &EntityRef, state: &State) -> Option<Self> {
        let vision_field = state
            .select::<(AnchorTransform, VisionField)>()
            .find(|(_, (anchor, _))| &anchor.0 == character)
            .map(|(e, _)| e)?;
        Some(Self {
            character: *character,
            vision_field,
        })
    }
}
