use std::fmt::Display;

/// A coordinate or extent in screen pixels.
///
/// Wraps the plain `f64` so pixel values cannot be confused with millimeter
/// positions. Also serves as the unit marker for `Vec2<Pixels>`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pixels(pub f64);

impl Pixels {
    /// Creates a pixel value.
    pub const fn new(pixels: f64) -> Self {
        Self(pixels)
    }

    /// The numeric value in pixels.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for Pixels {
    fn from(pixels: f64) -> Self {
        Self(pixels)
    }
}

impl From<Pixels> for f64 {
    fn from(pixels: Pixels) -> Self {
        pixels.0
    }
}

impl Display for Pixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
