use std::collections::VecDeque;

use crate::{
    controller::{ControlCommand, ControlDriver},
    prelude::*,
};

use super::{AiTask, AiTaskOutput};

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
        let front_task_output = self
            .tasks
            .pop_front()
            .map(|front_task| front_task.evaluate(actor, game_state));
        if let Some(front_task_output) = front_task_output {
            let mut issued_commands = Vec::new();
            let mut queued_tasks = Vec::new();
            front_task_output
                .into_iter()
                .for_each(|task_output| match task_output {
                    AiTaskOutput::QueueFront(task) => queued_tasks.push(task),
                    AiTaskOutput::IssueCmd(cmd) => issued_commands.push(cmd),
                });
            queued_tasks.into_iter().for_each(|queued_task| {
                self.tasks.push_front(queued_task);
            });
            issued_commands
        } else {
            vec![]
        }
    }
}
