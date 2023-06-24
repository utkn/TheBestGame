use crate::{
    character::CharacterInsights, controller::ControlCommand, item::EquipmentSlot,
    physics::ProjectileInsights, prelude::*, vehicle::VehicleInsights,
};

use super::{AiAttackHandler, AiMoveToPosHandler, AiTask, AiTaskHandler};

#[derive(Clone, Copy, Debug)]
pub struct AiRoutineHandler;

impl AiTaskHandler for AiRoutineHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask> {
        let insights = StateInsights::of(state);
        // Move towards the projectile.
        let hitter_vel = insights
            .new_hitters_of(actor)
            .into_iter()
            .next()
            .map(|(_, hit_vel)| hit_vel);
        if let Some((vx, vy)) = hitter_vel {
            let actor_trans = insights.transform_of(actor).unwrap();
            let rev_dir = notan::math::vec2(-*vx, -*vy).normalize();
            let target_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + rev_dir * 150.;
            return vec![
                AiTask::MoveToPos(AiMoveToPosHandler(target_pos.x, target_pos.y, true)),
                AiTask::Routine(self),
            ];
        }
        // Attack on sight.
        let visibles = insights.visibles_of_character(actor).unwrap_or_default();
        let target = visibles.into_iter().find(|e| {
            insights.is_character(e)
                || (insights.is_vehicle(e)
                    && insights
                        .driver_of(e)
                        .map(|driver| insights.is_character(driver))
                        .unwrap_or(false))
        });
        if let Some(target) = target {
            return vec![
                AiTask::Attack(AiAttackHandler::new(target)),
                AiTask::Routine(self),
            ];
        }
        vec![AiTask::Routine(self)]
    }

    fn get_commands(&self, actor: &EntityRef, state: &State) -> Vec<ControlCommand> {
        let insights = StateInsights::of(state);
        // Turn towards the projectile.
        let hitter_vel = insights
            .new_hitters_of(actor)
            .into_iter()
            .next()
            .map(|(_, hit_vel)| hit_vel);
        if let Some((vx, vy)) = hitter_vel {
            let rev_dir = notan::math::vec2(-*vx, -*vy).normalize();
            let target_deg = rev_dir
                .angle_between(notan::math::vec2(1., 0.))
                .to_degrees();
            return vec![
                ControlCommand::SetTargetVelocity(0., 0.),
                ControlCommand::SetTargetRotation(target_deg),
                ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand),
            ];
        }
        vec![
            ControlCommand::SetTargetVelocity(0., 0.),
            ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand),
        ]
    }
}
