use crate::{controller::ControlCommand, prelude::*};

mod ai_handlers;
mod ai_helpers;

use ai_handlers::*;

/// Represents what an [`AiTask`] can output.
#[derive(Clone, Debug)]
pub enum AiTaskOutput {
    /// Pushes a new [`AiTask`] into the front of the task queue.
    QueueFront(AiTask),
    /// Returns a [`ControlCommand`] to the driver.
    IssueCmd(ControlCommand),
}

/// Represents the possible tasks of an ai actor.
#[derive(Clone, Debug)]
pub enum AiTask {
    /// Actor attacks the target.
    Attack { target: EntityRef },
    /// Actor tries to move to the given position. The movement can be interrupted.
    TryMoveToPos {
        x: f32,
        y: f32,
        /// Denotes whether the actor should try to scale obstacles while performing the movement.
        /// If set to false, the movement is simply cancelled.
        scale_obstacles: bool,
    },
    /// Ai tries to move to the given position. The movement must be completed.
    MoveToPos { x: f32, y: f32 },
    /// Ai tries to get itself unstuck.
    TryScaleObstacle,
    /// Routine actions of the ai.
    Routine,
}

impl AiTask {
    /// Given the current state of the game, returns a list of [`AiTaskOutput`]s to be handled by the caller.
    pub fn evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
        // Dispatch to the correct handler.
        match self {
            AiTask::Attack { target } => attack_handler(target, actor, state),
            AiTask::TryMoveToPos {
                x,
                y,
                scale_obstacles,
            } => try_move_to_pos_handler(x, y, scale_obstacles, actor, state),
            AiTask::MoveToPos { x, y } => move_to_pos_handler(x, y, actor, state),
            AiTask::TryScaleObstacle => try_scale_obstacle_handler(actor, state),
            AiTask::Routine => routine_handler(actor, state),
        }
    }
}
