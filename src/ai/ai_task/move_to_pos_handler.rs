use crate::{
    controller::ControlCommand, item::EquipmentSlot, physics::ColliderInsights, prelude::*,
};

use super::{AiRoutineHandler, AiScaleObstacleHandler, AiTask, AiTaskHandler};

#[derive(Clone, Debug)]
pub struct AiMoveToPosHandler(pub f32, pub f32, pub bool);

impl AiMoveToPosHandler {
    fn get_dpos(&self, actor: &EntityRef, state: &State) -> (f32, f32) {
        let insights = StateInsights::of(state);
        let actor_pos = insights.transform_of(actor).unwrap();
        (self.0 - actor_pos.x, self.1 - actor_pos.y)
    }

    fn reached_destination(&self, actor: &EntityRef, state: &State) -> bool {
        let dpos = self.get_dpos(actor, state);
        dpos.0.abs() <= 5. && dpos.1.abs() <= 5.
    }
}

impl AiTaskHandler for AiMoveToPosHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        let routine_re_eval = AiRoutineHandler.re_evaluate(actor, state);
        if let [task1, AiTask::Routine(_), ..] = routine_re_eval.as_slice() {
            return vec![task1.clone()];
        }
        if self.reached_destination(actor, state) {
            return vec![];
        }
        let insights = StateInsights::of(state);
        if self.2 && insights.concrete_contacts_of(actor).len() > 0 {
            if self.2 {
                return vec![
                    AiTask::ScaleObstacle(AiScaleObstacleHandler),
                    AiTask::MoveToPos(self),
                ];
            } else {
                return vec![AiTask::ScaleObstacle(AiScaleObstacleHandler)];
            }
        }
        vec![AiTask::MoveToPos(self)]
    }

    fn get_commands(&self, actor: &EntityRef, state: &State) -> Vec<ControlCommand> {
        if self.reached_destination(actor, state) {
            return vec![ControlCommand::SetTargetVelocity(0., 0.)];
        }
        let dpos = self.get_dpos(actor, state);
        let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
        let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
        let target_vel = dir * 300.;
        vec![
            ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand),
            ControlCommand::SetTargetRotation(target_deg),
            ControlCommand::SetTargetVelocity(target_vel.x, target_vel.y),
        ]
    }
}
