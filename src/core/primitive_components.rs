use super::EntityRef;

#[derive(Clone, Copy, Default, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn translated(self, translation: (f32, f32)) -> Position {
        Position {
            x: self.x + translation.0,
            y: self.y + translation.1,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rotation {
    pub deg: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TargetRotation {
    pub deg: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TargetVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Acceleration(pub f32);

#[derive(Clone, Copy, Default, Debug)]
pub struct Controller {
    pub max_speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AnchorPosition(pub EntityRef, pub (f32, f32));

#[derive(Clone, Copy, Debug)]
pub struct Name(pub &'static str);
