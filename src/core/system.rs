use super::{State, StateCommands};

/// Represents the current state of the controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlMap {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub start_interact_pressed: bool,
    pub end_interact_pressed: bool,
}

impl ControlMap {
    pub fn from_app_state(app: &notan::prelude::App) -> Self {
        Self {
            left_pressed: app.keyboard.is_down(notan::prelude::KeyCode::A),
            right_pressed: app.keyboard.is_down(notan::prelude::KeyCode::D),
            up_pressed: app.keyboard.is_down(notan::prelude::KeyCode::W),
            down_pressed: app.keyboard.is_down(notan::prelude::KeyCode::S),
            start_interact_pressed: app.keyboard.was_pressed(notan::prelude::KeyCode::E),
            end_interact_pressed: app.keyboard.was_pressed(notan::prelude::KeyCode::Escape),
        }
    }
}

/// Contains the context information for an update iteration.
#[derive(Clone, Copy, Debug, Default)]
pub struct UpdateContext {
    pub dt: f32,
    pub control_map: ControlMap,
}

pub trait System: 'static {
    /// The update function for the system. This is called at every update iteration on the registered systems.
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands);
}
