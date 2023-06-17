use crate::{
    core::{primitive_components::*, EntityRef, StateCommands},
    equipment::{Equipment, EquipmentSlot, Equippable},
    interaction::{Interactable, InteractionType, ProximityInteractor},
    item::Item,
    physics::{CollisionState, Hitbox, HitboxType, Shape},
    storage::Storage,
};

use crate::core::EntityTemplate;

pub fn create_player(cmds: &mut StateCommands, pos: Position) -> EntityRef {
    (
        pos,
        Velocity::default(),
        Acceleration(2000.),
        TargetVelocity::default(),
        Controller { max_speed: 300. },
        Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
        CollisionState::default(),
        ProximityInteractor::default(),
        Storage::default(),
        Equipment::default(),
    )
        .create(cmds)
}

pub fn create_chest(cmds: &mut StateCommands, pos: Position) -> EntityRef {
    let chest_entity = (
        pos,
        Hitbox(HitboxType::Ghost, Shape::Rect(40., 40.)),
        CollisionState::default(),
        Interactable::new(InteractionType::ContactRequired),
        Storage::default(),
    )
        .create(cmds);
    let _chest_hitbox_entity = (
        Position::default(),
        Hitbox(HitboxType::Static, Shape::Rect(20., 20.)),
        AnchorPosition(chest_entity, (10., 10.)),
    )
        .create(cmds);
    chest_entity
}

pub fn create_item(cmds: &mut StateCommands, pos: Position, name: Name) -> EntityRef {
    (
        pos,
        name,
        Hitbox(HitboxType::Ghost, Shape::Circle(10.)),
        CollisionState::default(),
        Interactable::new(InteractionType::ContactRequiredOneShot),
        Item,
        Equippable::new([EquipmentSlot::Head]),
    )
        .create(cmds)
}
