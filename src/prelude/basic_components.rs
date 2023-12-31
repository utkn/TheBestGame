use super::EntityRef;

/// Represent the transformation of an entity.
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

    /// Returns the transform with the given degree.
    pub fn with_deg(self, deg: f32) -> Self {
        Self { deg, ..self }
    }

    /// Returns the transform with the given position.
    pub fn with_pos(self, x: f32, y: f32) -> Self {
        Self { x, y, ..self }
    }

    /// Returns a transform by applying the given translation.
    pub fn translated(self, translation: (f32, f32)) -> Transform {
        Transform {
            x: self.x + translation.0,
            y: self.y + translation.1,
            ..self
        }
    }

    /// Returns a unit size vector in the direction.
    pub fn dir_vec(&self) -> (f32, f32) {
        let (y, x) = self.deg.to_radians().sin_cos();
        (x, -y) // y-axis is inverted!
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

/// Represents the velocity of a component.
#[derive(Clone, Copy, Default, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// Represents the velocity that an entity wishes to achieve.
#[derive(Clone, Copy, Default, Debug)]
pub struct TargetVelocity {
    pub x: f32,
    pub y: f32,
}

/// Represents the rotation that an entity wishes to achieve.
#[derive(Clone, Copy, Default, Debug)]
pub struct TargetRotation {
    pub deg: f32,
}

/// Represents the maximum speed achievable by an entity.
#[derive(Clone, Copy, Default, Debug)]
pub struct MaxSpeed(pub f32);

/// Represents the acceleration of an entity. Used to determine the rate in which [`Velocity`] will be brought closer to [`TargetVelocity`].
#[derive(Clone, Copy, Default, Debug)]
pub struct Acceleration(pub f32);

/// The [`Transform`] of the entities with this component will be fixed to the [`Transform`] of the given parent with an optional offset.
#[derive(Clone, Copy, Debug)]
pub struct AnchorTransform(pub EntityRef, pub (f32, f32), pub f32);

/// A name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Name(pub &'static str);

/// Entities with this component will be removed after a period of time.
#[derive(Clone, Copy, Debug)]
pub struct Lifetime {
    pub remaining_time: f32,
}
