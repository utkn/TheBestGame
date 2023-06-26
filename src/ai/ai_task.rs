use crate::{controller::ControlCommand, prelude::*};

mod ai_handlers;
mod ai_helpers;
mod ai_movement;

use ai_handlers::*;
use ai_movement::*;

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
    /// Actor tries to move to the given position.
    MoveToPos(AiMovementHandler),
    /// Routine actions of the ai.
    Routine,
}

impl AiTask {
    /// Given the current state of the game, returns a list of [`AiTaskOutput`]s to be handled by the caller.
    pub fn evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
        // Dispatch to the correct handler.
        match self {
            AiTask::Attack { target } => attack_handler(target, actor, state),
            AiTask::Routine => routine_handler(actor, state),
            AiTask::MoveToPos(handler) => handler.handle(actor, state),
        }
    }
}
