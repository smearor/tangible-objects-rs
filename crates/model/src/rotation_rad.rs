/// A rotation angle in radians.
///
/// Wraps the plain `f64` so angles cannot be confused with distances or
/// other scalar values flowing through the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RotationRad(pub f64);

impl RotationRad {
    /// Creates a rotation from radians.
    pub const fn new(radians: f64) -> Self {
        Self(radians)
    }

    /// The angle in radians.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// The angle converted to degrees.
    pub fn to_degrees(self) -> f64 {
        self.0.to_degrees()
    }
}

impl From<f64> for RotationRad {
    fn from(radians: f64) -> Self {
        Self(radians)
    }
}

impl From<RotationRad> for f64 {
    fn from(rotation: RotationRad) -> Self {
        rotation.0
    }
}
