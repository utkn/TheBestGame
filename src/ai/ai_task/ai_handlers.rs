use crate::character::CharacterBundle;
use crate::{controller::ControlCommand, item::EquipmentSlot, prelude::*};

use super::ai_helpers::*;
use super::{AiMovementHandler, AiTask, AiTaskOutput};

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
    let ai_character = state
        .read_bundle::<CharacterBundle>(actor)
        .expect("ai actor is not a character!");
    let insights = StateInsights::of(state);
    // Handle the case that we cannot see the target anymore.
    if !ai_character.can_see(&target, state) {
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
                AiTaskOutput::QueueFront(AiTask::MoveToPos(AiMovementHandler::new(last_seen_pos))),
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
    let mut priority_actions = get_urgent_actions(actor, state);
    priority_actions.insert(0, AiTaskOutput::QueueFront(AiTask::Routine));
    priority_actions
}
