use crate::{
    activation::Activatable,
    core::*,
    effects::{Affected, Effector, EffectorTarget},
    equipment::{Equipment, EquipmentSlot, Equippable, SlotSelector},
    interaction::{HandInteractor, Interactable, InteractionDelegate, ProximityInteractor},
    item::Item,
    needs::{NeedMutator, NeedMutatorEffect, NeedStatus, NeedType, Needs},
    physics::{CollisionState, Hitbox, HitboxType, Shape},
    projectile::{ProjectileDefn, ProjectileGenerator},
    riding::Ridable,
    storage::Storage,
};

pub fn create_vehicle(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    let vehicle_entity = cmds.create_from((
        trans,
        Ridable,
        Velocity::default(),
        TargetVelocity::default(),
        Acceleration(2000.),
        MaxSpeed(800.),
        Interactable::default(),
        CollisionState::default(),
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
    ));
    vehicle_entity
}

pub fn create_character(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    cmds.create_from((
        trans,
        Velocity::default(),
        Acceleration(2000.),
        TargetVelocity::default(),
        MaxSpeed(300.),
        Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
        CollisionState::default(),
        Storage::default(),
        Equipment::default(),
        Needs::new([
            (NeedType::Health, NeedStatus::with_max(100.)),
            (NeedType::Sanity, NeedStatus::with_max(100.)),
            (NeedType::Hunger, NeedStatus::with_zero(100.)),
            (NeedType::Thirst, NeedStatus::with_zero(100.)),
        ]),
    ))
}

pub fn create_player(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    let character = create_character(trans, cmds);
    cmds.set_components(
        &character,
        (
            HandInteractor,
            ProximityInteractor,
            Controller { default_speed: 5. },
            FaceMouse,
            Affected::<MaxSpeed>::default(),
            Affected::<Acceleration>::default(),
        ),
    );
    character
}

pub fn create_chest(trans: Transform, cmds: &mut StateCommands) -> EntityRef {
    let _chest_center_entity =
        cmds.create_from((trans, Hitbox(HitboxType::Static, Shape::Rect(20., 20.))));
    let chest_entity = cmds.create_from((
        Transform::default(),
        Name("Some random chest"),
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
        CollisionState::default(),
        Interactable::default(),
        Activatable::<Storage>::default(),
        Storage::default(),
        AnchorTransform(_chest_center_entity, (-10., -10.)),
    ));
    chest_entity
}

pub fn create_item(
    trans: Transform,
    name: Name,
    slots: SlotSelector,
    cmds: &mut StateCommands,
) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        Item,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Equippable(slots),
        Interactable::default(),
    ))
}

pub fn create_handgun(trans: Transform, name: Name, cmds: &mut StateCommands) -> EntityRef {
    let item = create_item(
        trans,
        name,
        SlotSelector::new([[EquipmentSlot::LeftHand, EquipmentSlot::RightHand]]),
        cmds,
    );
    cmds.set_components(
        &item,
        (
            Storage::default(),
            Activatable::<Storage>::default(),
            Activatable::<ProjectileGenerator>::default(),
            ProjectileGenerator {
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
            Storage::default(),
            Activatable::<Storage>::default(),
            Activatable::<ProjectileGenerator>::default(),
            ProjectileGenerator {
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
        trans,
        name,
        SlotSelector::new([[EquipmentSlot::Feet]]),
        cmds,
    );
    cmds.set_components(
        &item,
        (
            Effector::<MaxSpeed>::new([EffectorTarget::Equipper], |old| MaxSpeed(old.0 * 2.)),
            Effector::<Acceleration>::new([EffectorTarget::Equipper], |old| {
                Acceleration(old.0 * 4.)
            }),
        ),
    );
    item
}
