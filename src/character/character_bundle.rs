use std::collections::HashSet;

use crate::{item::*, needs::*, physics::*, prelude::*};

use super::Character;
use crate::item::EquipmentInsights;

#[derive(Clone, Copy, Debug)]
pub struct CharacterBundle {
    pub character: EntityRef,
    pub vision_field: EntityRef,
    pub collision_senser: EntityRef,
}

impl CharacterBundle {
    pub fn create(trans: Transform, cmds: &mut StateCommands) -> Self {
        let character = cmds.create_from((
            trans,
            Character,
            Velocity::default(),
            Acceleration(2000.),
            TargetVelocity::default(),
            TargetRotation::default(),
            MaxSpeed(300.),
            Hitbox(HitboxType::Dynamic, Shape::Rect { w: 20., h: 20. }),
            InteractTarget::<Hitbox>::default(),
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
        let vf_radius = 200.;
        let vision_field = cmds.create_from((
            Transform::default(),
            AnchorTransform(character, (vf_radius, 0.)),
            Hitbox(HitboxType::Ghost, Shape::Circle { r: vf_radius }),
            InteractTarget::<Hitbox>::default(),
            VisionField(vf_radius),
        ));
        let collision_senser = cmds.create_from((
            Transform::default(),
            AnchorTransform(character, (0., 0.)),
            Hitbox(HitboxType::Ghost, Shape::Rect { w: 30., h: 30. }),
            InteractTarget::<Hitbox>::default(),
        ));
        cmds.push_bundle(Self {
            character,
            vision_field,
            collision_senser,
        })
    }

    /// Returns true if this character can see the given entity `other`.
    pub fn can_see(&self, other: &EntityRef, state: &State) -> bool {
        self.visibles(state).contains(other)
    }

    /// Returns true if this character can see the given entity `other`.
    pub fn visibles(&self, state: &State) -> HashSet<EntityRef> {
        StateInsights::of(state).visibles_of(&self.vision_field)
    }

    pub fn get_backpack<'a>(&self, state: &'a State) -> Option<&'a EntityRef> {
        StateInsights::of(state)
            .equippable_at(&self.character, &EquipmentSlot::Backpack)
            .and_then(|item_stack| item_stack.head_item())
    }

    pub fn concrete_overlaps<'a>(&self, state: &'a State) -> Vec<&'a (f32, f32)> {
        state
            .read_events::<CollisionEvt>()
            .filter(|evt| &evt.e1 == &self.collision_senser)
            .filter(|evt| &evt.e2 != &self.character)
            .filter(|evt| {
                state
                    .select_one::<(Hitbox,)>(&evt.e2)
                    .map(|(hb,)| hb.0.is_concrete())
                    .unwrap_or(false)
            })
            .map(|evt| &evt.overlap)
            .collect()
    }

    pub fn is_colliding(&self, state: &State) -> bool {
        self.concrete_overlaps(state).len() >= 1
    }
}

impl<'a> EntityBundle<'a> for CharacterBundle {
    type TupleRepr = (EntityRef, EntityRef, EntityRef);

    fn primary_entity(&self) -> &EntityRef {
        &self.character
    }

    fn deconstruct(self) -> Self::TupleRepr {
        (self.character, self.vision_field, self.collision_senser)
    }

    fn reconstruct(args: <Self::TupleRepr as EntityTuple<'a>>::AsRefTuple) -> Self {
        Self {
            character: *args.0,
            vision_field: *args.1,
            collision_senser: *args.2,
        }
    }
}
