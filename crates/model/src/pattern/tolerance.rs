use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;

/// Matching tolerance for a figure pattern.
///
/// Maximum allowed deviation per pairwise distance during fingerprint
/// matching. Serializes as a bare number so existing profile files stay
/// compatible.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct PatternTolerance(pub Millimeters);

impl PatternTolerance {
    /// Creates a tolerance.
    pub const fn new(millimeters: f64) -> Self {
        Self(Millimeters::new(millimeters))
    }

    /// The tolerance value.
    pub const fn as_millimeters(self) -> Millimeters {
        self.0
    }

    /// The numeric tolerance value.
    pub const fn as_f64(self) -> f64 {
        self.0.as_f64()
    }
}

impl From<f64> for PatternTolerance {
    fn from(millimeters: f64) -> Self {
        Self(Millimeters::new(millimeters))
    }
}

impl From<Millimeters> for PatternTolerance {
    fn from(millimeters: Millimeters) -> Self {
        Self(millimeters)
    }
}

impl From<PatternTolerance> for Millimeters {
    fn from(tolerance: PatternTolerance) -> Self {
        tolerance.0
    }
}

impl From<PatternTolerance> for f64 {
    fn from(tolerance: PatternTolerance) -> Self {
        tolerance.0.as_f64()
    }
}

impl Display for PatternTolerance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
