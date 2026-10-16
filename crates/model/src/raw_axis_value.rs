use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Div;
use std::ops::Sub;

use crate::UnitsPerMm;

/// Raw axis value reported by the device (bounds of `ABS_MT_POSITION_X/Y`).
///
/// Wraps the plain `f64` so raw device coordinates cannot be confused with
/// millimeter positions or resolutions. Dividing by [`UnitsPerMm`] yields
/// millimeters as `f64`. Serializes as a bare number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RawAxisValue(pub f64);

impl RawAxisValue {
    /// Creates a raw axis value from its numeric value.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// The raw numeric axis value.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for RawAxisValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<RawAxisValue> for f64 {
    fn from(value: RawAxisValue) -> Self {
        value.0
    }
}

impl Sub for RawAxisValue {
    type Output = RawAxisValue;

    fn sub(self, rhs: RawAxisValue) -> RawAxisValue {
        Self(self.0 - rhs.0)
    }
}

impl Div<UnitsPerMm> for RawAxisValue {
    type Output = f64;

    fn div(self, rhs: UnitsPerMm) -> f64 {
        self.0 / rhs.0
    }
}

impl Display for RawAxisValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
