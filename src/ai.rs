use std::collections::VecDeque;

use crate::{
    character::{CharacterBundle, CharacterInsights},
    controller::{ControlCommand, ControlDriver},
    item::EquipmentSlot,
    prelude::*,
};

use itertools::Itertools;
pub use vision_field::*;
pub use vision_insights::*;

mod vision_field;
mod vision_insights;

#[derive(Clone, Copy, Debug)]
enum AiTask {
    Attack(EntityRef),
    Follow(EntityRef, f32),
    Routine,
}

impl AiTask {
    fn get_commands(&self, ai_actor: &EntityRef, state: &State) -> Vec<ControlCommand> {
        match self {
            AiTask::Attack(target) => {
                if let Some(dpos) = StateInsights::of(state).pos_diff(target, ai_actor) {
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
            AiTask::Follow(target, _) => {
                if let Some(dpos) = StateInsights::of(state).pos_diff(target, ai_actor) {
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
            AiTask::Routine => vec![ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand)],
        }
    }

    fn generate(self, ai_actor: &EntityRef, state: &State) -> Vec<AiTask> {
        match self {
            AiTask::Attack(target) => {
                if !state.is_valid(&target) {
                    return vec![];
                }
                let ai_char = CharacterBundle::try_reconstruct(ai_actor, state).unwrap();
                let insights = StateInsights::of(state);
                let ai_visibles = insights.visibles_of(&ai_char.vision_field);
                if !ai_visibles.contains(&target) {
                    return vec![];
                }
                let too_far_away = StateInsights::of(state)
                    .dist_sq_between(ai_actor, &target)
                    .map(|dst_sq| dst_sq > 150. * 150.)
                    .unwrap_or(false);
                if too_far_away {
                    vec![AiTask::Follow(target, 150.), self]
                } else {
                    vec![self]
                }
            }
            AiTask::Follow(target, min_dist) => {
                if !state.is_valid(&target) {
                    return vec![];
                }
                let ai_char = CharacterBundle::try_reconstruct(ai_actor, state).unwrap();
                let insights = StateInsights::of(state);
                let ai_visibles = insights.visibles_of(&ai_char.vision_field);
                if !ai_visibles.contains(&target) {
                    return vec![];
                }
                if let Some(dist_sq) = StateInsights::of(state).dist_sq_between(ai_actor, &target) {
                    // println!("{:?} {:?}", min_dist * min_dist, dist_sq);
                    if min_dist * min_dist < dist_sq {
                        vec![self]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            AiTask::Routine => {
                let ai_vision_field = CharacterBundle::try_reconstruct(ai_actor, state)
                    .map(|bundle| bundle.vision_field);
                if let Some(ai_vision_field) = ai_vision_field {
                    let insights = StateInsights::of(state);
                    let visibles = insights.visibles_of(&ai_vision_field);
                    if let Some(target) = visibles
                        .iter()
                        .find(|visible| insights.is_character(visible))
                    {
                        vec![AiTask::Attack(*target), self]
                    } else {
                        vec![self]
                    }
                } else {
                    vec![self]
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AiDriver {
    tasks: VecDeque<AiTask>,
}

impl Default for AiDriver {
    fn default() -> Self {
        Self {
            tasks: VecDeque::from_iter([AiTask::Routine]),
        }
    }
}

impl ControlDriver for AiDriver {
    fn get_commands(
        &mut self,
        actor: &EntityRef,
        _ctx: &UpdateContext,
        game_state: &State,
    ) -> Vec<ControlCommand> {
        // Try to extend the task queue.
        let new_front_tasks = self
            .tasks
            .pop_front()
            .map(|front_task| front_task.generate(actor, game_state));
        if let Some(new_front_tasks) = new_front_tasks {
            new_front_tasks.into_iter().rev().for_each(|new_task| {
                self.tasks.push_front(new_task);
            });
        }
        // println!("{:?}", self.tasks);
        if let Some(first) = self.tasks.front() {
            first.get_commands(actor, game_state)
        } else {
            vec![]
        }
    }
}
