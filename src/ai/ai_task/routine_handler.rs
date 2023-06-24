use crate::{
    ai::ai_task::attack_handler::AiAttackHandler,
    character::{CharacterBundle, CharacterInsights},
    controller::ControlCommand,
    item::EquipmentSlot,
    physics::VisionInsights,
    prelude::*,
};

use super::{AiTask, AiTaskHandler};

#[derive(Clone, Copy, Debug)]
pub struct AiRoutineHandler;

impl AiTaskHandler for AiRoutineHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        let ai_vision_field =
            CharacterBundle::try_reconstruct(actor, state).map(|bundle| bundle.vision_field);
        let insights = StateInsights::of(state);
        if let Some(ai_vision_field) = ai_vision_field {
            let visibles = insights.visibles_of(&ai_vision_field);
            if let Some(target) = visibles
                .iter()
                .find(|visible| insights.is_character(visible))
            {
                vec![
                    AiTask::Attack(AiAttackHandler { target: *target }),
                    AiTask::Routine(self),
                ]
            } else {
                vec![AiTask::Routine(self)]
            }
        } else {
            vec![AiTask::Routine(self)]
        }
    }

    fn get_commands(&self, _actor: &EntityRef, _state: &State) -> Vec<ControlCommand> {
        vec![ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand)]
    }
}
