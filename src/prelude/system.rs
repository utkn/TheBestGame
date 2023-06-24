use super::{State, StateCommands};

/// Represents the current state of the controller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ControlMap {
    pub left_is_down: bool,
    pub right_is_down: bool,
    pub up_is_down: bool,
    pub down_is_down: bool,
    pub start_interact_was_pressed: bool,
    pub end_interact_was_pressed: bool,
    pub mouse_left_was_pressed: bool,
    pub mouse_right_was_pressed: bool,
    pub mouse_left_was_released: bool,
    pub mouse_right_was_released: bool,
    pub mouse_left_is_down: bool,
    pub mouse_right_is_down: bool,
    pub mouse_pos: (f32, f32),
}

impl ControlMap {
    pub fn from_app_state(app: &notan::prelude::App) -> Self {
        Self {
            left_is_down: app.keyboard.is_down(notan::prelude::KeyCode::A),
            right_is_down: app.keyboard.is_down(notan::prelude::KeyCode::D),
            up_is_down: app.keyboard.is_down(notan::prelude::KeyCode::W),
            down_is_down: app.keyboard.is_down(notan::prelude::KeyCode::S),
            start_interact_was_pressed: app.keyboard.was_pressed(notan::prelude::KeyCode::E),
            end_interact_was_pressed: app.keyboard.was_pressed(notan::prelude::KeyCode::Escape),
            mouse_pos: app.mouse.position(),
            mouse_left_was_pressed: app.mouse.left_was_pressed(),
            mouse_right_was_pressed: app.mouse.right_was_pressed(),
            mouse_left_was_released: app.mouse.left_was_released(),
            mouse_right_was_released: app.mouse.right_was_released(),
            mouse_left_is_down: app.mouse.left_is_down(),
            mouse_right_is_down: app.mouse.right_is_down(),
        }
    }
}

/// Contains the context information for an update iteration.
#[derive(Clone, Copy, Debug, Default)]
pub struct UpdateContext {
    pub dt: f32,
    pub control_map: ControlMap,
}

pub trait System: 'static + std::fmt::Debug {
    /// The update function for the system. This is called at every update iteration on the registered systems.
    fn update(&mut self, ctx: &UpdateContext, state: &State, cmds: &mut StateCommands);
}
