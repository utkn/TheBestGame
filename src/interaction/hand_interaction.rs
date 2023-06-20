use crate::{
    core::*,
    equipment::{Equipment, EquipmentSlot},
};

use super::{TryInteractReq, TryUninteractReq};

/// An actor that can interact with what they have on their hands (i.e., in their appropriate equipment slot).
#[derive(Clone, Copy, Debug)]
pub struct HandInteractor;

/// A system that handles the entities that can interact with their equipment.
#[derive(Clone, Copy, Debug)]
pub struct HandInteractionSystem;

impl System for HandInteractionSystem {
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state
            .select::<(HandInteractor, Equipment)>()
            .for_each(|(e, (_, equipment))| {
                // Get the left & right hand items of the hand interactor actor.
                let (lh_item, rh_item) = (
                    equipment.get(EquipmentSlot::LeftHand),
                    equipment.get(EquipmentSlot::RightHand),
                );
                // If left mouse is pressed, try to interact with the left hand item.
                if ctx.control_map.mouse_left_was_pressed {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryInteractReq::new(e, *lh_item));
                    }
                }
                if ctx.control_map.mouse_left_was_released {
                    if let Some(lh_item) = lh_item {
                        cmds.emit_event(TryUninteractReq::new(e, *lh_item));
                    }
                }
                // If right mouse is pressed, try to interact with the right hand item.
                if ctx.control_map.mouse_right_was_pressed {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryInteractReq::new(e, *rh_item));
                    }
                }
                if ctx.control_map.mouse_right_was_released {
                    if let Some(rh_item) = rh_item {
                        cmds.emit_event(TryUninteractReq::new(e, *rh_item));
                    }
                }
            })
    }
}
