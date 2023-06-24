use std::collections::VecDeque;

use crate::{
    controller::{ControlCommand, ControlDriver},
    prelude::*,
};

use super::{AiRoutineHandler, AiTask};

#[derive(Clone, Debug)]
pub struct AiDriver {
    tasks: VecDeque<AiTask>,
}

impl Default for AiDriver {
    fn default() -> Self {
        Self {
            tasks: VecDeque::from_iter([AiTask::Routine(AiRoutineHandler)]),
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
            .map(|front_task| front_task.re_evaluate(actor, game_state));
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
