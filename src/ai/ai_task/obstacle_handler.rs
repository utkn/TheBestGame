use rand::Rng;

use super::{AiTask, AiTaskHandler};
use crate::{controller::ControlCommand, physics::ColliderInsights, prelude::*};

use super::AiMoveToPosHandler;

#[derive(Clone, Copy, Debug)]
pub struct AiScaleObstacleHandler;

impl AiTaskHandler for AiScaleObstacleHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        let insights = StateInsights::of(state);
        if let Some(overlap) = insights.concrete_contact_overlaps_of(actor).first() {
            let actor_trans = insights.transform_of(actor).unwrap();
            let mut dev = rand::thread_rng().gen_range(45_f32..80_f32);
            if rand::random() {
                dev *= -1.
            }
            let side_dir = notan::math::Vec2::from_angle(dev.to_radians())
                .rotate(notan::math::vec2(-overlap.0, -overlap.1));
            let new_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + side_dir * 40.;
            vec![AiTask::MoveToPos(AiMoveToPosHandler(
                new_pos.x, new_pos.y, false,
            ))]
        } else {
            vec![]
        }
    }

    fn get_commands(&self, _actor: &EntityRef, _state: &State) -> Vec<ControlCommand> {
        vec![ControlCommand::SetTargetVelocity(0., 0.)]
    }
}
