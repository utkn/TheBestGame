use crate::{item::*, needs::*, physics::*, prelude::*};

use super::{Character, CharacterInsights};

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
        Self {
            character,
            vision_field,
        }
    }

    fn primary_entity(&self) -> &EntityRef {
        &self.character
    }

    fn try_reconstruct(character: &EntityRef, state: &State) -> Option<Self> {
        let vision_field = StateInsights::of(state).vision_field_of(character)?;
        Some(Self {
            character: *character,
            vision_field,
        })
    }
}
