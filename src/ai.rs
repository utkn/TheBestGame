use crate::{
    controller::{ControlCommand, ControlDriver},
    prelude::*,
};

#[derive(Clone, Copy, Debug)]
pub struct AiDriver;

impl ControlDriver for AiDriver {
    type State = ();

    fn get_command(
        &self,
        _actor: &EntityRef,
        _ctx: &UpdateContext,
        _game_state: &State,
        _driver_state: &mut Self::State,
    ) -> ControlCommand {
        ControlCommand::None
    }
}
