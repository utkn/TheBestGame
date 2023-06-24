use crate::{
    ai::*,
    camera::CameraFollow,
    character::create_character,
    controller::{Controller, ProximityInteractable, UserInputDriver},
    effects::{Affected, Effect, Effector, EffectorTarget},
    item::*,
    needs::{NeedMutator, NeedMutatorEffect, NeedType},
    physics::*,
    prelude::*,
};

pub fn create_player(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
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
    character
}

pub fn create_chest(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
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
    chest
}

pub fn create_handgun(trans: Transform, name: Name, cmds: &mut StateCommands) -> EntityRef {
    let item = create_item(
        Item::stackable(2),
        trans,
        name,
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
    item
}
pub fn create_machinegun(trans: Transform, name: Name, cmds: &mut StateCommands) -> EntityRef {
    let item = create_item(
        Item::unstackable(),
        trans,
        name,
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
    item
}

pub fn create_shoes(trans: Transform, name: Name, cmds: &mut StateCommands) -> EntityRef {
    let item = create_item(
        Item::unstackable(),
        trans,
        name,
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
    item
}
