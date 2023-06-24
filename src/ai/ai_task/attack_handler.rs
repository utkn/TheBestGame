use crate::{
    character::CharacterBundle, controller::ControlCommand, item::EquipmentSlot,
    physics::VisionInsights, prelude::*,
};

use super::{AiFollowHandler, AiTask, AiTaskHandler};

#[derive(Clone, Copy, Debug)]
pub struct AiAttackHandler {
    pub target: EntityRef,
}

impl AiTaskHandler for AiAttackHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        if !state.is_valid(&self.target) {
            return vec![];
        }
        let ai_char = CharacterBundle::try_reconstruct(actor, state).unwrap();
        let insights = StateInsights::of(state);
        let ai_visibles = insights.visibles_of(&ai_char.vision_field);
        if !ai_visibles.contains(&self.target) {
            return vec![];
        }
        let too_far_away = StateInsights::of(state)
            .dist_sq_between(actor, &self.target)
            .map(|dst_sq| dst_sq > 150. * 150.)
            .unwrap_or(false);
        if too_far_away {
            vec![
                AiTask::Follow(AiFollowHandler {
                    target: self.target,
                    min_dist: 150.,
                }),
                AiTask::Attack(self),
            ]
        } else {
            vec![AiTask::Attack(self)]
        }
    }

    fn get_commands(
        &self,
        actor: &EntityRef,
        state: &crate::prelude::State,
    ) -> Vec<crate::controller::ControlCommand> {
        if let Some(dpos) = StateInsights::of(state).pos_diff(&self.target, actor) {
            if dpos.0 == 0. && dpos.1 == 0. {
                return vec![];
            }
            let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
            let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
            vec![
                ControlCommand::EquipmentInteract(EquipmentSlot::LeftHand),
                ControlCommand::SetTargetRotation(target_deg),
                ControlCommand::SetTargetVelocity(0., 0.),
            ]
        } else {
            vec![]
        }
    }
}
