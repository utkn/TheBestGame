use crate::{
    activation::{Activatable, ActivationLoc},
    core::*,
    effects::{Affected, Effector, EffectorTarget},
    equipment::{Equipment, EquipmentSlot, Equippable},
    interaction::{HandInteractor, Interactable, InteractionType, ProximityInteractor},
    item::Item,
    needs::{NeedMutator, NeedMutatorTarget, NeedStatus, NeedType, Needs},
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
        MaxSpeed(300.),
        Controller { default_speed: 5. },
        Hitbox(HitboxType::Dynamic, Shape::Rect(20., 20.)),
        CollisionState::default(),
        ProximityInteractor::default(),
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
        &player_entity,
        (
            HandInteractor,
            FaceMouse,
            Affected::<MaxSpeed>::default(),
            Affected::<Acceleration>::default(),
        ),
    );
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
        NeedMutator::new([NeedMutatorTarget::Storage], NeedType::Sanity, -5.),
        Effector::<MaxSpeed>::new([EffectorTarget::Equipper], |old| MaxSpeed(old.0 * 2.)),
        Effector::<Acceleration>::new([EffectorTarget::Equipper], |old| Acceleration(old.0 * 4.)),
    ))
}
