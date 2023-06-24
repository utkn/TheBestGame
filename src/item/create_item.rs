use crate::{
    ai::VisionField, controller::ProximityInteractable, physics::*, prelude::*, sprite::Sprite,
};

use super::{Equippable, Item, SlotSelector};

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
        Sprite::new("generic_item", 0),
    ))
}
