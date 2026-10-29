use serde::Deserialize;
use serde::Serialize;
use std::f64::consts::PI;

use crate::SmoothingAlpha;

/// A rotation angle in radians.
///
/// Wraps the plain `f64` so angles cannot be confused with distances or
/// other scalar values flowing through the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
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

    /// `(sin, cos)` of the angle - trigonomtrie for transforms and
    /// geometry (the `f64::sin_cos` delegate).
    pub fn sin_cos(self) -> (f64, f64) {
        self.0.sin_cos()
    }

    /// Wraparound-aware EMA blend - `old + alpha * diff(new, old)`,
    /// where `diff` is the shortest signed arc between the angles
    /// (modulo 2π). `alpha` is the weight of `new`.
    pub fn ema(old: RotationRad, new: RotationRad, alpha: SmoothingAlpha) -> RotationRad {
        let mut diff = new.0 - old.0;
        while diff > PI {
            diff -= 2.0 * PI;
        }
        while diff < -PI {
            diff += 2.0 * PI;
        }
        Self(old.0 + alpha.as_f64() * diff)
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

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(RotationRad);
