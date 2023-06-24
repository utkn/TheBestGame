use crate::{
    character::CharacterBundle,
    controller::ControlCommand,
    item::EquipmentSlot,
    physics::{ColliderInsights, VisionInsights},
    prelude::*,
};

use super::{AiScaleObstacleHandler, AiTask, AiTaskHandler};

#[derive(Clone, Copy, Debug)]
pub struct AiFollowHandler {
    pub target: EntityRef,
    pub min_dist: f32,
}

impl AiTaskHandler for AiFollowHandler {
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
        if let Some(dist_sq) = StateInsights::of(state).dist_sq_between(actor, &self.target) {
            // println!("{:?} {:?}", min_dist * min_dist, dist_sq);
            if self.min_dist * self.min_dist < dist_sq {
                vec![AiTask::Follow(self)]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn get_commands(
        &self,
        actor: &EntityRef,
        state: &State,
    ) -> Vec<crate::controller::ControlCommand> {
        if let Some(dpos) = StateInsights::of(state).pos_diff(&self.target, actor) {
            if dpos.0 == 0. && dpos.1 == 0. {
                return vec![];
            }
            let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
            let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
            let target_vel = dir * 300.;
            vec![
                ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand),
                ControlCommand::SetTargetRotation(target_deg),
                ControlCommand::SetTargetVelocity(target_vel.x, target_vel.y),
            ]
        } else {
            vec![]
        }
    }
}
