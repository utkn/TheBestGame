use crate::{
    ai::*,
    camera::CameraFollow,
    controller::{Controller, UserInputDriver},
    effects::{Affected, Effect, Effector, EffectorTarget},
    item::*,
    needs::{NeedMutator, NeedMutatorEffect, NeedStatus, NeedType, Needs},
    physics::*,
    prelude::*,
    vehicle::Vehicle,
};

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
        Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
        InteractTarget::<Hitbox>::default(),
        InteractTarget::<VisionField>::default(),
    ));
    cmds.set_components(
        &vehicle,
        (
            Name("vehicle"),
            Equipment::new([EquipmentSlot::VehicleGas, EquipmentSlot::VehicleModule]),
            InteractTarget::<Equipment>::default(),
        ),
    );
    let _vehicle_door = cmds.create_from((
        Transform::default(),
        AnchorTransform(vehicle, (0., 0.)),
        ProximityInteractable,
        UntargetedInteractionDelegate(vehicle),
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
        InteractTarget::<Hitbox>::default(),
        ExistenceDependency(vehicle),
    ));
    vehicle
}

pub fn create_character(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
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
    let _character_vision_field = cmds.create_from((
        Transform::default(),
        AnchorTransform(character, (0., 0.)),
        Hitbox(HitboxType::Ghost, Shape::Circle(50.)),
        InteractTarget::<Hitbox>::default(),
        VisionField(50.),
    ));
    character
}

pub fn create_player(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    let character = create_character(trans, cmds);
    cmds.set_components(
        &character,
        (
            HandInteractor,
            CameraFollow,
            Controller(UserInputDriver { default_speed: 5. }),
            FaceMouse,
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

pub fn create_item(
    item: Item,
    trans: Transform,
    name: Name,
    slots: SlotSelector,
    cmds: &mut StateCommands,
) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        item,
        ProximityInteractable,
        InteractTarget::<Item>::default(),
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        InteractTarget::<Hitbox>::default(),
        Equippable(slots),
        InteractTarget::<VisionField>::default(),
    ))
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
                cooldown: Some(0.1),
                proj: ProjectileDefn {
                    lifetime: 1.5,
                    speed: 600.,
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
