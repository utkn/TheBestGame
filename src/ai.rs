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

impl ControlDriver for AiDriver {
    type State = ();

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
