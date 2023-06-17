use crate::{
    core::*,
    equipment::{Equipment, EquipmentSlot},
    interaction::{Interaction, InteractionEndedEvt, InteractionStartedEvt, TryInteractReq},
};

#[derive(Clone, Copy, Default, Debug)]
pub struct Activatable(pub bool);

#[derive(Clone, Copy, Debug)]
pub struct ActivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct DeactivatedEvt(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct ActivationSystem;

impl System for ActivationSystem {
    fn update(&mut self, _: &UpdateContext, state: &State, cmds: &mut StateCommands) {
        state.read_events::<InteractionEndedEvt>().for_each(|evt| {
            if let Some((target_activ,)) = state.select_one::<(Activatable,)>(&evt.0.target) {
                if target_activ.0 {
                    cmds.emit_event(DeactivatedEvt(evt.0.target));
                    cmds.update_component(&evt.0.target, |activ: &mut Activatable| {
                        activ.0 = false;
                    })
                }
            }
        });
        state
            .read_events::<InteractionStartedEvt>()
            .for_each(|evt| {
                if let Some((target_activ,)) = state.select_one::<(Activatable,)>(&evt.0.target) {
                    if !target_activ.0 {
                        cmds.emit_event(ActivatedEvt(evt.0.target));
                        cmds.update_component(&evt.0.target, |activ: &mut Activatable| {
                            activ.0 = true;
                        })
                    }
                }
            });
    }
}
