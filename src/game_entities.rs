use crate::{
    activation::{Activatable, ActivationLoc},
    core::*,
    equipment::{Equipment, EquipmentSlot, Equippable},
    interaction::{HandInteractor, Interactable, InteractionType, ProximityInteractor},
    item::Item,
    needs::{NeedStatus, NeedType, Needs},
    physics::{CollisionState, Hitbox, HitboxType, Shape},
    projectile::{ProjectileDefn, ProjectileGenerator},
    storage::Storage,
};

pub fn create_player(cmds: &mut StateCommands, trans: Transform) -> EntityRef {
    let player_entity = cmds.create_from((
        trans,
        Velocity::default(),
        Acceleration(2000.),
        TargetVelocity::default(),
        Controller { max_speed: 300. },
        Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
        CollisionState::default(),
        ProximityInteractor::default(),
        HandInteractor,
        Storage::default(),
        Equipment::default(),
        Needs::new([
            (NeedType::Health, NeedStatus::with_max(100)),
            (NeedType::Sanity, NeedStatus::with_max(100)),
            (NeedType::Hunger, NeedStatus::with_zero(100)),
            (NeedType::Thirst, NeedStatus::with_zero(100)),
        ]),
    ));
    cmds.set_components(&player_entity, (FaceMouse,));
    player_entity
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

pub fn create_item(cmds: &mut StateCommands, trans: Transform, name: Name) -> EntityRef {
    cmds.create_from((
        trans,
        name,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Interactable::new(InteractionType::ContactRequiredOneShot),
        Item,
        Equippable::new([EquipmentSlot::LeftHand]),
        Activatable::at_locations([ActivationLoc::Equipment]),
        ProjectileGenerator(ProjectileDefn {
            lifetime: 0.5,
            speed: 300.,
        }),
    ))
}
