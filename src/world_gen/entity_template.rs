use crate::{
    ai::AiDriver, camera::CameraFollow, character::CharacterBundle, controller::*, effects::*,
    item::*, needs::*, physics::*, prelude::*, sprite::Sprite, vehicle::VehicleBundle,
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
        let character = CharacterBundle::create(trans, cmds);
        cmds.set_components(
            character.primary_entity(),
            (
                Sprite::new("player", 3),
                CameraFollow,
                Controller(UserInputDriver),
                Affected::<MaxSpeed>::default(),
                Affected::<Acceleration>::default(),
            ),
        );
        // cmds.mark_for_removal(&character.vision_field);
        Some(*character.primary_entity())
    },
};

pub const BANDIT_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, state, cmds| {
        let character = CharacterBundle::create(trans, cmds);
        cmds.set_components(
            character.primary_entity(),
            (
                Sprite::new("bandit", 3),
                Controller(AiDriver::default()),
                Affected::<MaxSpeed>::default(),
                Affected::<Acceleration>::default(),
            ),
        );
        let bandit_weapon = MACHINE_GUN_TEMPLATE.generate(trans, state, cmds)?;
        cmds.emit_event(ItemTransferReq::equip_from_ground(
            bandit_weapon,
            *character.primary_entity(),
        ));
        Some(*character.primary_entity())
    },
};

pub const CHEST_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| {
        let storage_bundle = StorageBundle::create(trans, cmds);
        cmds.set_component(storage_bundle.primary_entity(), Name("some random chest"));
        Some(*storage_bundle.primary_entity())
    },
};

pub const BASIC_CAR_TEMPLATE: EntityTemplate = EntityTemplate {
    generator: |trans, _state, cmds| Some(*VehicleBundle::create(trans, cmds).primary_entity()),
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
                    auto_knockback: None,
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
                    auto_knockback: Some(100.),
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
