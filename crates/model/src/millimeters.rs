use std::fmt::Display;

/// A length or coordinate in millimeters on the touch surface.
///
/// Wraps the plain `f64` so millimeter values cannot be confused with raw
/// device units or screen pixels. Also serves as the unit marker for
/// `Vec2<Millimeters>`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Millimeters(pub f64);

impl Millimeters {
    /// Creates a millimeter value.
    pub const fn new(millimeters: f64) -> Self {
        Self(millimeters)
    }

    /// The numeric value in millimeters.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for Millimeters {
    fn from(millimeters: f64) -> Self {
        Self(millimeters)
    }
}

impl From<Millimeters> for f64 {
    fn from(millimeters: Millimeters) -> Self {
        millimeters.0
    }
}

impl Display for Millimeters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
