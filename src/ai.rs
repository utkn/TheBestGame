use crate::{
    controller::{ControlCommand, ControlDriver},
    core::*,
};

#[derive(Clone, Copy, Debug)]
pub struct AiDriver;

impl ControlDriver for AiDriver {
    type State = ();

    fn get_command(
        &self,
        actor: &EntityRef,
        ctx: &UpdateContext,
        game_state: &State,
        driver_state: &mut Self::State,
    ) -> ControlCommand {
        ControlCommand::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fraction {
    GoodGuy,
    BadGuy,
    AntiHero,
}

#[derive(Clone, Copy, Debug)]
pub struct Vision;
