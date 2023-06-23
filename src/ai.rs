use crate::{
    controller::{ControlCommand, ControlDriver},
    prelude::*,
};

pub use vision_field::*;
pub use vision_insights::*;

mod vision_field;
mod vision_insights;

#[derive(Clone, Copy, Debug)]
pub struct AiDriver;

#[derive(Clone, Default, Debug)]
pub struct AiState {
    curr_follow_target: Option<EntityRef>,
}

impl ControlDriver for AiDriver {
    type State = AiState;

    fn get_commands(
        &self,
        _actor: &EntityRef,
        _ctx: &UpdateContext,
        _game_state: &State,
        _driver_state: &mut Self::State,
    ) -> Vec<ControlCommand> {
        todo!()
    }
}
