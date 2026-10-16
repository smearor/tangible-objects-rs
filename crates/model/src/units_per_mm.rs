use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Device resolution: device units per millimeter on an axis.
///
/// Wraps the `resolution` field of evdev `ABS_INFO` so resolutions cannot be
/// confused with raw axis values or millimeter distances. Serializes as a
/// bare number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitsPerMm(pub f64);

impl UnitsPerMm {
    /// Creates a resolution from units per millimeter.
    pub const fn new(units_per_mm: f64) -> Self {
        Self(units_per_mm)
    }

    /// The resolution in units per millimeter.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for UnitsPerMm {
    fn from(units_per_mm: f64) -> Self {
        Self(units_per_mm)
    }
}

impl From<UnitsPerMm> for f64 {
    fn from(units_per_mm: UnitsPerMm) -> Self {
        units_per_mm.0
    }
}

impl Display for UnitsPerMm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
