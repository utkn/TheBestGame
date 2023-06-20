use super::EntityRef;

#[derive(Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    /// Rotation in degrees.
    pub deg: f32,
}

impl Transform {
    /// Creates a new transform at the given position.
    pub fn at(x: f32, y: f32) -> Self {
        Self { x, y, deg: 0. }
    }

    /// Returns a transform with the given degree.
    pub fn with_deg(self, deg: f32) -> Self {
        Self { deg, ..self }
    }

    /// Returns a transform by applying the given translation.
    pub fn translated(self, translation: (f32, f32)) -> Transform {
        Transform {
            x: self.x + translation.0,
            y: self.y + translation.1,
            deg: self.deg,
        }
    }
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
pub struct MaxSpeed(pub f32);

#[derive(Clone, Copy, Default, Debug)]
pub struct TargetVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Acceleration(pub f32);

#[derive(Clone, Copy, Default, Debug)]
pub struct Controller {
    pub default_speed: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct FaceMouse;

#[derive(Clone, Copy, Debug)]
pub struct AnchorTransform(pub EntityRef, pub (f32, f32));

#[derive(Clone, Copy, Debug)]
pub struct ExistenceDependency(pub EntityRef);

#[derive(Clone, Copy, Debug)]
pub struct Name(pub &'static str);

#[derive(Clone, Copy, Debug)]
pub struct Lifetime {
    pub remaining_time: f32,
}
