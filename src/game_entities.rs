use crate::{
    activation::{Activatable, ActivationLoc},
    core::*,
    effects::{Affected, Effector, EffectorTarget},
    equipment::{Equipment, EquipmentSlot, Equippable},
    interaction::{HandInteractor, Interactable, InteractionType, ProximityInteractor},
    item::Item,
    needs::{NeedMutatorEffect, NeedStatus, NeedType, Needs},
    physics::{CollisionState, Hitbox, HitboxType, Shape},
    projectile::{ProjectileDefn, ProjectileGenerator},
    storage::Storage,
};

pub fn create_player(cmds: &mut StateCommands, trans: Transform) -> EntityRef {
    let character_entity = cmds.create_from((
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
    ));
    cmds.set_components(
        &character_entity,
        (
            HandInteractor,
            ProximityInteractor,
            Controller { default_speed: 5. },
            FaceMouse,
            Affected::<MaxSpeed>::default(),
            Affected::<Acceleration>::default(),
        ),
    );
    character_entity
}

pub fn create_chest(cmds: &mut StateCommands, trans: Transform) -> EntityRef {
    let chest_entity = cmds.create_from((
        trans,
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
        CollisionState::default(),
        Interactable::new(InteractionType::ContactRequired),
        Storage::default(),
    ));
    let _chest_hitbox_entity = cmds.create_from((
        Transform::default(),
        Hitbox(HitboxType::Static, Shape::Rect(20., 20.)),
        AnchorTransform(chest_entity, (10., 10.)),
    ));
    chest_entity
}

pub fn create_handgun(cmds: &mut StateCommands, trans: Transform, name: Name) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        Item,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Interactable::new(InteractionType::OneShot),
        Equippable::new([EquipmentSlot::LeftHand]),
        Activatable::at_locations([ActivationLoc::Equipment]),
        ProjectileGenerator {
            cooldown: None,
            proj: ProjectileDefn {
                lifetime: 0.5,
                speed: 300.,
                need_mutation: (NeedType::Health, NeedMutatorEffect::Delta(-5.)),
            },
        },
    ))
}
pub fn create_machinegun(cmds: &mut StateCommands, trans: Transform, name: Name) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        Item,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Interactable::new(InteractionType::Whatevs),
        Equippable::new([EquipmentSlot::RightHand]),
        Activatable::at_locations([ActivationLoc::Equipment]),
        ProjectileGenerator {
            cooldown: Some(0.1),
            proj: ProjectileDefn {
                lifetime: 1.5,
                speed: 600.,
                need_mutation: (NeedType::Health, NeedMutatorEffect::Delta(-5.)),
            },
        },
    ))
}

pub fn create_shoes(cmds: &mut StateCommands, trans: Transform, name: Name) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        Item,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Interactable::new(InteractionType::ContactRequiredOneShot),
        Equippable::new([EquipmentSlot::Feet]),
        Effector::<MaxSpeed>::new([EffectorTarget::Equipper], |old| MaxSpeed(old.0 * 2.)),
        Effector::<Acceleration>::new([EffectorTarget::Equipper], |old| Acceleration(old.0 * 4.)),
    ))
}
