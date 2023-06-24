use crate::{controller::ControlCommand, prelude::*};

mod attack_handler;
mod follow_handler;
mod routine_handler;

pub use attack_handler::*;
pub use follow_handler::*;
pub use routine_handler::*;

#[derive(Clone, Debug)]
pub enum AiTask {
    Attack(AiAttackHandler),
    Follow(AiFollowHandler),
    Routine(AiRoutineHandler),
}

impl AiTask {
    pub fn get_commands(&self, ai_actor: &EntityRef, state: &State) -> Vec<ControlCommand> {
        match self {
            AiTask::Attack(handler) => handler.get_commands(ai_actor, state),
            AiTask::Follow(handler) => handler.get_commands(ai_actor, state),
            AiTask::Routine(handler) => handler.get_commands(ai_actor, state),
        }
    }

    pub fn re_evaluate(self, ai_actor: &EntityRef, state: &State) -> Vec<AiTask> {
        match self {
            AiTask::Attack(handler) => handler.re_evaluate(ai_actor, state),
            AiTask::Follow(handler) => handler.re_evaluate(ai_actor, state),
            AiTask::Routine(handler) => handler.re_evaluate(ai_actor, state),
        }
    }
}

pub trait AiTaskHandler {
    fn re_evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTask>;
    fn get_commands(&self, actor: &EntityRef, state: &State) -> Vec<ControlCommand>;
}
