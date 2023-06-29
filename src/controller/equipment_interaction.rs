use crate::{
    item::{Equipment, EquipmentSlot},
    prelude::*,
};

use super::{TryInteractReq, TryUninteractReq};

#[derive(Clone, Copy, Debug)]
pub struct EquipmentInteractReq(pub EntityRef, pub EquipmentSlot);

#[derive(Clone, Copy, Debug)]
pub struct EquipmentUninteractReq(pub EntityRef, pub EquipmentSlot);

/// A system that handles the entities that can interact with their equipment.
#[derive(Clone, Copy, Debug)]
pub struct EquipmentInteractionSystem;

impl<R: StateReader> System<R> for EquipmentInteractionSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &R, cmds: &mut StateCommands) {
        state.read_events::<EquipmentInteractReq>().for_each(|evt| {
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.0) {
                let item_at_slot = equipment
                    .get_item_stack(&evt.1)
                    .and_then(|item_slot| item_slot.head_item());
                if let Some(item) = item_at_slot {
                    cmds.emit_event(TryInteractReq::new(evt.0, *item));
                }
            }
        });
        state
            .read_events::<EquipmentUninteractReq>()
            .for_each(|evt| {
                if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.0) {
                    let item_at_slot = equipment
                        .get_item_stack(&evt.1)
                        .and_then(|item_slot| item_slot.head_item());
                    if let Some(item) = item_at_slot {
                        cmds.emit_event(TryUninteractReq::new(evt.0, *item));
                    }
                }
            });
    }
}
