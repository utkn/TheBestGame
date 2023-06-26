use itertools::Itertools;
use std::collections::VecDeque;

use notan::math;

use crate::{
    ai::MovementGrid,
    character::{CharacterBundle, CharacterInsights},
    controller::ControlCommand,
    physics::{ColliderInsights, EffectiveHitbox, Hitbox},
    prelude::*,
};

use super::{ai_helpers::*, AiTask, AiTaskOutput};

#[derive(Clone, Debug)]
pub struct AiMovementHandler {
    path_to_follow: VecDeque<(f32, f32)>,
    target: (f32, f32),
}

impl AiMovementHandler {
    pub fn new(target: (f32, f32)) -> Self {
        Self {
            path_to_follow: Default::default(),
            target,
        }
    }

    pub fn handle(mut self, actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
        if get_urgent_actions(actor, state).len() > 0 {
            return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                0., 0.,
            ))];
        }
        if reached_destination_approx(&self.target.0, &self.target.1, actor, state) {
            return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                0., 0.,
            ))];
        }
        if self.path_to_follow.is_empty() {
            if let Some(path) = compute_path(&self.target, actor, state) {
                self.path_to_follow = path;
                return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos(self))];
            } else {
                return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                    0., 0.,
                ))];
            }
        }
        let next_milestone = self.path_to_follow.front().unwrap();
        if reached_destination_approx(&next_milestone.0, &next_milestone.1, actor, state) {
            self.path_to_follow.pop_front();
            return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos(self))];
        }
        let dpos = get_dpos(&next_milestone.0, &next_milestone.1, actor, state).unwrap();
        let speed = state
            .select_one::<(MaxSpeed,)>(actor)
            .map(|(max_speed,)| max_speed.0)
            .unwrap_or(50.);
        let target_dir = math::vec2(dpos.0, dpos.1).normalize_or_zero();
        let target_deg = target_dir.angle_between(math::vec2(1., 0.)).to_degrees();
        return vec![
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetRotation(target_deg)),
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                target_dir.x * speed,
                target_dir.y * speed,
            )),
            AiTaskOutput::QueueFront(AiTask::MoveToPos(self)),
        ];
    }
}

fn compute_path(
    target: &(f32, f32),
    actor: &EntityRef,
    state: &State,
) -> Option<VecDeque<(f32, f32)>> {
    let cell_size = {
        state
            .select_one::<(Hitbox,)>(actor)
            .map(|(hb,)| match hb.1 {
                crate::physics::Shape::Circle { r } => 2. * r,
                crate::physics::Shape::Rect { w, h } => w.max(h),
            })? as isize
    };
    let (actor_trans,) = state.select_one::<(Transform,)>(actor)?;
    let mut movement_grid = MovementGrid::new(cell_size, &(actor_trans.x, actor_trans.y), target);
    let actor_char = state.read_bundle::<CharacterBundle>(actor)?;
    let hitboxes_in_range = StateInsights::of(state)
        .contacts_of(&actor_char.vision_field)
        .unwrap()
        .into_iter()
        .filter(|e| !StateInsights::of(state).is_character(&e))
        .flat_map(|e| state.select_one::<(Hitbox,)>(e).map(|(hb,)| (e, hb)))
        .filter(|(_, hb)| hb.0.is_concrete())
        .flat_map(|(e, _)| EffectiveHitbox::new(&e, state))
        .collect_vec();
    movement_grid.fill_obstructions(&hitboxes_in_range);
    movement_grid.find_path()
}
