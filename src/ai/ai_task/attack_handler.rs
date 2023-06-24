use crate::{
    character::CharacterInsights, controller::ControlCommand, item::EquipmentSlot, prelude::*,
};

use super::{AiFollowHandler, AiMoveToPosHandler, AiTask, AiTaskHandler};

#[derive(Clone, Copy, Debug)]
pub struct AiAttackHandler {
    pub target: EntityRef,
}

impl AiAttackHandler {
    pub fn new(target: EntityRef) -> Self {
        Self { target }
    }
}

impl AiTaskHandler for AiAttackHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        if !state.is_valid(&self.target) {
            return vec![];
        }
        let insights = StateInsights::of(state);
        // Handle the case that we cannot see the target anymore.
        let actor_visibles = insights.visibles_of_character(actor).unwrap_or_default();
        if !actor_visibles.contains(&self.target) {
            let last_seen_pos = insights
                .transform_of(&self.target)
                .map(|trans| (trans.x, trans.y));
            if let Some(last_seen_pos) = last_seen_pos {
                return vec![AiTask::MoveToPos(AiMoveToPosHandler(
                    last_seen_pos.0,
                    last_seen_pos.1,
                    true,
                ))];
            } else {
                return vec![];
            }
        }
        let min_dist = 250.;
        let too_far_away = insights
            .dist_sq_between(actor, &self.target)
            .map(|dst_sq| dst_sq > min_dist * min_dist)
            .unwrap_or(false);
        if too_far_away {
            vec![
                AiTask::Follow(AiFollowHandler {
                    target: self.target,
                    min_dist,
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
