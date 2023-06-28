use crate::prelude::*;

/// The entity tagged by this component will be followed by the camera.
#[derive(Clone, Copy, Debug)]
pub struct CameraFollow {
    pub w: f32,
    pub h: f32,
}

impl CameraFollow {
    pub fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }
}

/// Converts from world coordinates to the screen coordinates.
pub fn map_to_screen_cords(
    world_x: f32,
    world_y: f32,
    screen_width: f32,
    screen_height: f32,
    state: &State,
) -> (f32, f32) {
    if let Some((_, (_, trans))) = state.select::<(CameraFollow, Transform)>().next() {
        (
            world_x - trans.x + screen_width / 2.,
            world_y - trans.y + screen_height / 2.,
        )
    } else {
        (world_x, world_y)
    }
}

/// Converts from screen coordinates to world coordinates.
pub fn map_to_world_cords(
    screen_x: f32,
    screen_y: f32,
    screen_width: f32,
    screen_height: f32,
    state: &State,
) -> (f32, f32) {
    if let Some((_, (_, trans))) = state.select::<(CameraFollow, Transform)>().next() {
        (
            screen_x + trans.x - screen_width / 2.,
            screen_y + trans.y - screen_height / 2.,
        )
    } else {
        (screen_x, screen_y)
    }
}
