use crate::{
    ai::VisionField, camera::CameraFollow, character::create_character, controller::*, effects::*,
    item::*, needs::*, physics::*, prelude::*, vehicle::create_vehicle,
};

pub struct EntityTemplate {
    generator: fn(trans: Transform, state: &State, cmds: &mut StateCommands) -> Option<EntityRef>,
}

impl EntityTemplate {
    pub(super) fn generate(
        &self,
        trans: Transform,
        state: &State,
        cmds: &mut StateCommands,
    ) -> Option<EntityRef> {
        (self.generator)(trans, state, cmds)
    }
}

pub const PLAYER_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let character = create_character("player", trans, cmds);
        cmds.set_components(
            &character,
            (
                CameraFollow,
                Controller(UserInputDriver),
                Affected::<MaxSpeed>::default(),
                Affected::<Acceleration>::default(),
            ),
        );
        Some(character)
    },
};

pub const CHEST_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let chest = cmds.create_from((
            trans,
            Name("Some random chest"),
            Hitbox(HitboxType::Static, Shape::Rect(20., 20.)),
            InteractTarget::<Hitbox>::default(),
            InteractTarget::<Storage>::default(),
            Storage::new(60),
            InteractTarget::<VisionField>::default(),
        ));
        let _chest_activator = cmds.create_from((
            Transform::default(),
            AnchorTransform(chest, (0., 0.)),
            ProximityInteractable,
            UntargetedInteractionDelegate(chest),
            Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
            InteractTarget::<Hitbox>::default(),
            ExistenceDependency(chest),
        ));
        Some(chest)
    },
};

pub const BASIC_CAR_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| Some(create_vehicle(trans, cmds)),
};

pub const HAND_GUN_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let item = create_item(
            Item::stackable(2),
            trans,
            Name("HandGun"),
            SlotSelector::new([[EquipmentSlot::LeftHand, EquipmentSlot::RightHand]]),
            cmds,
        );
        cmds.set_components(
            &item,
            (
                Storage::new(100),
                InteractTarget::<Storage>::default(),
                InteractTarget::<ProjectileGenerator>::default(),
                ProjectileGenerator {
                    knockback: None,
                    cooldown: None,
                    proj: ProjectileDefn {
                        lifetime: 0.5,
                        speed: 300.,
                        spread: 0.,
                        on_hit: NeedMutator::new(NeedType::Health, NeedMutatorEffect::Delta(-5.)),
                    },
                },
            ),
        );
        Some(item)
    },
};

pub const MACHINE_GUN_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let item = create_item(
            Item::unstackable(),
            trans,
            Name("MachineGun"),
            SlotSelector::new([
                [EquipmentSlot::LeftHand, EquipmentSlot::RightHand],
                [EquipmentSlot::LeftHand, EquipmentSlot::RightHand],
            ]),
            cmds,
        );
        cmds.set_components(
            &item,
            (
                Storage::new(100),
                InteractTarget::<Storage>::default(),
                InteractTarget::<ProjectileGenerator>::default(),
                ProjectileGenerator {
                    knockback: Some(100.),
                    cooldown: Some(0.05),
                    proj: ProjectileDefn {
                        lifetime: 1.5,
                        speed: 2000.,
                        spread: 15.,
                        on_hit: NeedMutator::new(NeedType::Health, NeedMutatorEffect::Delta(-5.)),
                    },
                },
            ),
        );
        Some(item)
    },
};
pub const RUNNING_SHOES_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let item = create_item(
            Item::unstackable(),
            trans,
            Name("RunningShoes"),
            SlotSelector::new([[EquipmentSlot::Feet]]),
            cmds,
        );
        cmds.set_components(
            &item,
            (
                Effector::<MaxSpeed>::new([EffectorTarget::Equipper], Effect::Multiply(2.)),
                Effector::<Acceleration>::new([EffectorTarget::Equipper], Effect::Multiply(4.)),
            ),
        );
        Some(item)
    },
};
