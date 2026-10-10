use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Matching tolerance for a figure pattern, in millimeters.
///
/// Maximum allowed deviation per pairwise distance during fingerprint
/// matching. Serializes as a bare number so existing profile files stay
/// compatible.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PatternTolerance(pub f64);

impl PatternTolerance {
    /// Creates a tolerance in millimeters.
    pub const fn new(millimeters: f64) -> Self {
        Self(millimeters)
    }

    /// The tolerance in millimeters.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for PatternTolerance {
    fn from(millimeters: f64) -> Self {
        Self(millimeters)
    }
}

impl From<PatternTolerance> for f64 {
    fn from(tolerance: PatternTolerance) -> Self {
        tolerance.0
    }
}

impl Display for PatternTolerance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
