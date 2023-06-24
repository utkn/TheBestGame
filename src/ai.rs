use crate::{
    controller::{ControlCommand, ControlDriver},
    prelude::*,
};

pub use vision_field::*;
pub use vision_insights::*;

mod vision_field;
mod vision_insights;

#[derive(Clone, Copy, Debug)]
pub enum AiTask {
    FollowPersistent,
    Follow,
    Routine,
}

#[derive(Clone, Default, Copy, Debug)]
pub struct AiDriver {
    curr_follow_target: Option<EntityRef>,
}

impl ControlDriver for AiDriver {
    fn get_commands(
        &mut self,
        _actor: &EntityRef,
        _ctx: &UpdateContext,
        _game_state: &State,
    ) -> Vec<ControlCommand> {
        todo!()
    }
}
