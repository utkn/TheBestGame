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

/// A stateful handler for ai movement to a target position. Avoid obstacles & takes a shortest path.
#[derive(Clone, Debug)]
pub struct AiMovementHandler {
    /// The current path that the ai is following.
    path_to_follow: VecDeque<(f32, f32)>,
    /// Ultimate target.
    target: (f32, f32),
}

impl AiMovementHandler {
    pub fn new(target: (f32, f32)) -> Self {
        Self {
            path_to_follow: Default::default(),
            target,
        }
    }

    pub fn handle(mut self, actor: &EntityRef, state: &impl StateReader) -> Vec<AiTaskOutput> {
        // If there are urgent actions, cancel the movement.
        if get_urgent_actions(actor, state).len() > 0 {
            return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                0., 0.,
            ))];
        }
        // If we reached our destination, cancel the movement.
        if reached_destination_approx(&self.target.0, &self.target.1, actor, state) {
            return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                0., 0.,
            ))];
        }
        // If we have no path to follow (which will be the case initially), generate it.
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
        // Read the current milestone from the path that we are following.
        let next_milestone = self.path_to_follow.front().unwrap();
        // If we reached the current milestone, remove it from the path and queue itself.
        if reached_destination_approx(&next_milestone.0, &next_milestone.1, actor, state) {
            self.path_to_follow.pop_front();
            return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos(self))];
        }
        // Otherwise, approach to the current milestone.
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

/// Computes a shortest path from the `actor`'s position to the given `target` position.
fn compute_path(
    target: &(f32, f32),
    actor: &EntityRef,
    state: &impl StateReader,
) -> Option<VecDeque<(f32, f32)>> {
    // Dynamic cell size selection. A cell must fully encompass the hitbox of the actor.
    let cell_size = {
        state
            .select_one::<(Hitbox,)>(actor)
            .map(|(hb,)| match hb.1 {
                crate::physics::Shape::Circle { r } => 2. * r,
                crate::physics::Shape::Rect { w, h } => w.max(h),
            })? as isize
    };
    let actor_char = state.read_bundle::<CharacterBundle>(actor)?;
    // Read the hitboxes that we must try to avoid.
    let hitboxes_in_range = StateInsights::of(state)
        .contacts_of(&actor_char.vision_field)
        .unwrap()
        .into_iter()
        .filter(|e| !StateInsights::of(state).is_character(&e))
        .flat_map(|e| state.select_one::<(Hitbox,)>(e).map(|(hb,)| (e, hb)))
        .filter(|(_, hb)| hb.0.is_concrete())
        .flat_map(|(e, _)| EffectiveHitbox::new(&e, state))
        .collect_vec();
    let (actor_trans,) = state.select_one::<(Transform,)>(actor)?;
    let actor_pos = (actor_trans.x, actor_trans.y);
    // The movement grid will be constructed relative to the starting position. If the shortest paths require
    // movements smaller than the cell size, we may not be able to find any in the first iteration. Augment the
    // starting position and regenerate the grids to accomodate for this.
    let starting_positions = std::iter::once(actor_pos).chain(
        (1..cell_size).map(|delta| delta as f32).flat_map(|delta| {
            [
                (actor_pos.0, actor_pos.1 - delta),
                (actor_pos.0, actor_pos.1 + delta),
                (actor_pos.0 - delta, actor_pos.1),
                (actor_pos.0 - delta, actor_pos.1 - delta),
                (actor_pos.0 - delta, actor_pos.1 + delta),
                (actor_pos.0 + delta, actor_pos.1),
                (actor_pos.0 + delta, actor_pos.1 - delta),
                (actor_pos.0 + delta, actor_pos.1 + delta),
            ]
        }),
    );
    // Find the first successful shortest path.
    starting_positions
        .into_iter()
        .flat_map(|starting_pos| {
            // Generate the grid relative to the starting position.
            let mut movement_grid = MovementGrid::new(cell_size, &starting_pos, target);
            // Fill the obstructions by supplying the hitboxes `in range`.
            movement_grid.fill_obstructions(&hitboxes_in_range);
            // Try to find a path. If so, the first milestone in the path should be this starting position.
            movement_grid.find_path().map(|path| (starting_pos, path))
        })
        .next()
        .map(|(starting_pos, mut path)| {
            path.push_front(starting_pos);
            path
        })
}
