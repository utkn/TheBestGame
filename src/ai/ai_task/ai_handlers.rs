use rand::Rng;

use crate::{
    character::CharacterInsights, controller::ControlCommand, item::EquipmentSlot,
    physics::ColliderInsights, prelude::*,
};

use super::ai_helpers::*;
use super::{AiTask, AiTaskOutput};

pub(super) fn attack_handler(
    target: EntityRef,
    actor: &EntityRef,
    state: &State,
) -> Vec<AiTaskOutput> {
    // Cancel the attack if the target is no longer valid.
    if !state.is_valid(&target) {
        return vec![
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
            AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand)),
        ];
    }
    let insights = StateInsights::of(state);
    // Handle the case that we cannot see the target anymore.
    let can_see = insights
        .visibles_of_character(actor)
        .map(|visibles| visibles.contains(&target))
        .unwrap_or(false);
    if !can_see {
        // Get the last seen position of the target.
        let last_seen_pos = insights
            .transform_of(&target)
            .map(|trans| (trans.x, trans.y));
        // Start a movement to the last seen position of the target.
        if let Some(last_seen_pos) = last_seen_pos {
            return vec![
                AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
                AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(
                    EquipmentSlot::LeftHand,
                )),
                AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
                    x: last_seen_pos.0,
                    y: last_seen_pos.1,
                    scale_obstacles: true,
                }),
            ];
        } else {
            return vec![
                AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
                AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(
                    EquipmentSlot::LeftHand,
                )),
            ];
        }
    }
    // If we can see the target, keep attacking it by looking at it.
    let dpos = insights.pos_diff(&target, actor).unwrap_or_default();
    let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
    let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
    return vec![
        AiTaskOutput::QueueFront(AiTask::Attack { target }),
        AiTaskOutput::IssueCmd(ControlCommand::SetTargetRotation(target_deg)),
        AiTaskOutput::IssueCmd(ControlCommand::EquipmentInteract(EquipmentSlot::LeftHand)),
    ];
}

pub(super) fn routine_handler(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    let mut priority_actions = get_priority_actions(actor, state);
    priority_actions.insert(0, AiTaskOutput::QueueFront(AiTask::Routine));
    priority_actions
}

pub(super) fn move_to_pos_handler(
    x: f32,
    y: f32,
    actor: &EntityRef,
    state: &State,
) -> Vec<AiTaskOutput> {
    // Reached the destination. Stop moving and remove itself from the queue.
    if reached_destination(&x, &y, actor, state) {
        return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
            0., 0.,
        ))];
    }
    // Otherwise, keep trying to move to the target position.
    return vec![AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
        x,
        y,
        scale_obstacles: true,
    })];
}

pub(super) fn try_move_to_pos_handler(
    x: f32,
    y: f32,
    scale_obstacles: bool,
    actor: &EntityRef,
    state: &State,
) -> Vec<AiTaskOutput> {
    // Reached the destination. Stop moving and remove itself from the queue.
    if reached_destination(&x, &y, actor, state) {
        return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
            0., 0.,
        ))];
    }
    // Check whether the actor is stuck or not.
    let is_stuck = StateInsights::of(state).concrete_contacts_of(actor).len() > 0;
    // Try to find some urgent actions. Those actions take priority over completing the movement process.
    // First, get the priority actions, e.g., enemy on sight.
    let mut urgent_actions = get_priority_actions(actor, state);
    // If we have no priority actions and we encountered an obstacle, issue obstacle scaling as an urgent action. Afterall,
    // we cannot complete the movement if we are stuck.
    if urgent_actions.is_empty() && scale_obstacles && is_stuck {
        // Maintain itself.
        urgent_actions.push(AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
            x,
            y,
            scale_obstacles,
        }));
        urgent_actions.push(AiTaskOutput::QueueFront(AiTask::TryScaleObstacle));
    }
    if urgent_actions.len() > 0 {
        urgent_actions.push(AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
            0., 0.,
        )));
        return urgent_actions;
    }
    // Approach to the target by getting the delta position between the actor and the target position.
    if let Some(dpos) = get_dpos(&x, &y, actor, state) {
        let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
        let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
        // Get the speed of the actor.
        let speed = state
            .select_one::<(MaxSpeed,)>(actor)
            .map(|(max_speed,)| max_speed.0)
            .unwrap_or(100.);
        let target_vel = dir * speed;
        return vec![
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetRotation(target_deg)),
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                target_vel.x,
                target_vel.y,
            )),
            // Maintain itself
            AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
                x,
                y,
                scale_obstacles,
            }),
        ];
    }
    // End the task if delta pos couldn't be found.
    return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
        0., 0.,
    ))];
}

pub(super) fn try_scale_obstacle_handler(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    let insights = StateInsights::of(state);
    if let Some(overlap) = insights.concrete_contact_overlaps_of(actor).first() {
        let actor_trans = insights.transform_of(actor).unwrap();
        let mut dev = rand::thread_rng().gen_range(60_f32..80_f32);
        if rand::random() {
            dev *= -1.
        }
        let side_dir = notan::math::Vec2::from_angle(dev.to_radians())
            .rotate(notan::math::vec2(-overlap.0, -overlap.1));
        let new_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + side_dir * 40.;
        // Replace itself with a nonpersistent movement from the obstacle.
        return vec![AiTaskOutput::QueueFront(AiTask::TryMoveToPos {
            x: new_pos.x,
            y: new_pos.y,
            scale_obstacles: false,
        })];
    }
    return vec![];
}
