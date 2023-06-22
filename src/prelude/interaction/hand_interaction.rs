use crate::{
    item::{Equipment, EquipmentSlot},
    prelude::*,
};

use super::{TryInteractReq, TryUninteractReq};

/// An actor that can interact with what they have on their hands (i.e., in their appropriate equipment slot).
#[derive(Clone, Copy, Debug)]
pub struct HandInteractor;

#[derive(Clone, Copy, Debug)]
pub enum HandSide {
    Left,
    Right,
}

impl From<HandSide> for EquipmentSlot {
    fn from(value: HandSide) -> Self {
        match value {
            HandSide::Left => EquipmentSlot::LeftHand,
            HandSide::Right => EquipmentSlot::RightHand,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HandInteractReq(pub EntityRef, pub HandSide);

#[derive(Clone, Copy, Debug)]
pub struct HandUninteractReq(pub EntityRef, pub HandSide);

/// A system that handles the entities that can interact with their equipment.
#[derive(Clone, Copy, Debug)]
pub struct HandInteractionSystem;

/// TODO: integrate with the new controller system. Refactor as `EquipmentInteractorSystem` that listens to equipment interaction events.
impl System for HandInteractionSystem {
    fn update(&mut self, _ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<HandInteractReq>().for_each(|evt| {
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.0) {
                let item_at_slot = equipment
                    .get_item_stack(&evt.1.into())
                    .and_then(|item_slot| item_slot.head_item());
                if let Some(item) = item_at_slot {
                    cmds.emit_event(TryInteractReq::new(evt.0, *item));
                }
            }
        });
        state.read_events::<HandUninteractReq>().for_each(|evt| {
            if let Some((equipment,)) = state.select_one::<(Equipment,)>(&evt.0) {
                let item_at_slot = equipment
                    .get_item_stack(&evt.1.into())
                    .and_then(|item_slot| item_slot.head_item());
                if let Some(item) = item_at_slot {
                    cmds.emit_event(TryUninteractReq::new(evt.0, *item));
                }
            }
        });
    }
}
