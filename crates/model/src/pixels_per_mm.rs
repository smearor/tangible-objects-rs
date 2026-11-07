use crate::Millimeters;
use crate::Pixels;
use crate::Vec2;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Render scale: screen pixels per millimeter of surface.
///
/// The counterpart of [`crate::UnitsPerMm`] (device units → mm, sensor
/// calibration): this one maps millimeters → pixels, the *display*
/// side. Wrapping the bare `f64` keeps the two factors unconfusable -
/// a `PixelsPerMm` is never a resolution and never an axis bound.
/// Serializes as a bare number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PixelsPerMm(pub f64);

impl PixelsPerMm {
    /// Creates a scale from pixels per millimeter.
    pub const fn new(pixels_per_mm: f64) -> Self {
        Self(pixels_per_mm)
    }

    /// The scale in pixels per millimeter.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Maps a millimeter length onto pixels.
    pub fn scale(&self, mm: Millimeters) -> Pixels {
        Pixels::new(mm.as_f64() * self.0)
    }

    /// Maps a millimeter vector onto pixels, component-wise.
    pub fn scale_vec(&self, vector: Vec2<Millimeters>) -> Vec2<Pixels> {
        Vec2::<Pixels>::new(
            self.scale(Millimeters::new(vector.x)).as_f64(),
            self.scale(Millimeters::new(vector.y)).as_f64(),
        )
    }

    /// Maps a pixel length back onto millimeters - the hit-testing
    /// direction (widget geometry → surface coordinates).
    pub fn unscale(&self, px: Pixels) -> Millimeters {
        Millimeters::new(px.as_f64() / self.0)
    }

    /// Maps a pixel vector back onto millimeters, component-wise.
    pub fn unscale_vec(&self, vector: Vec2<Pixels>) -> Vec2<Millimeters> {
        Vec2::<Millimeters>::new(
            self.unscale(Pixels::new(vector.x)).as_f64(),
            self.unscale(Pixels::new(vector.y)).as_f64(),
        )
    }
}

impl From<f64> for PixelsPerMm {
    fn from(pixels_per_mm: f64) -> Self {
        Self(pixels_per_mm)
    }
}

impl From<PixelsPerMm> for f64 {
    fn from(pixels_per_mm: PixelsPerMm) -> Self {
        pixels_per_mm.0
    }
}

impl Display for PixelsPerMm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Millimeters;
    use super::Pixels;
    use super::PixelsPerMm;

    #[test]
    fn scale_and_unscale_are_inverses() {
        let factor = PixelsPerMm::new(4.0);
        assert!((factor.scale(Millimeters::new(10.0)).as_f64() - 40.0).abs() < 1e-9);
        assert!((factor.unscale(Pixels::new(40.0)).as_f64() - 10.0).abs() < 1e-9);
        assert!(
            (factor
                .unscale(factor.scale(Millimeters::new(17.5)))
                .as_f64()
                - 17.5)
                .abs()
                < 1e-9
        );
    }
}
