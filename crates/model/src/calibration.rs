use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;
use crate::Pixels;
use crate::RawAxisValue;
use crate::Region;
use crate::UnitsPerMm;
use crate::Vec2;

/// Device calibration: converts raw device coordinates into millimeters.
///
/// `x_res`/`y_res` are the `resolution` fields of `ABS_MT_POSITION_X/Y`
/// (units per millimeter) read from evdev `ABS_INFO` or an `evtest` header.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    /// Device units per millimeter on the X axis.
    pub x_res: UnitsPerMm,
    /// Device units per millimeter on the Y axis.
    pub y_res: UnitsPerMm,
    /// Minimum raw X value reported by the device.
    pub x_min: RawAxisValue,
    /// Maximum raw X value reported by the device.
    pub x_max: RawAxisValue,
    /// Minimum raw Y value reported by the device.
    pub y_min: RawAxisValue,
    /// Maximum raw Y value reported by the device.
    pub y_max: RawAxisValue,
}

impl Calibration {
    /// Creates a calibration without bounds information.
    pub fn new(x_res: impl Into<UnitsPerMm>, y_res: impl Into<UnitsPerMm>) -> Self {
        Self {
            x_res: x_res.into(),
            y_res: y_res.into(),
            x_min: RawAxisValue::new(0.0),
            x_max: RawAxisValue::new(0.0),
            y_min: RawAxisValue::new(0.0),
            y_max: RawAxisValue::new(0.0),
        }
    }

    /// Creates a calibration including the device's raw axis bounds.
    pub fn with_bounds(
        x_res: impl Into<UnitsPerMm>,
        y_res: impl Into<UnitsPerMm>,
        x_min: impl Into<RawAxisValue>,
        x_max: impl Into<RawAxisValue>,
        y_min: impl Into<RawAxisValue>,
        y_max: impl Into<RawAxisValue>,
    ) -> Self {
        Self {
            x_res: x_res.into(),
            y_res: y_res.into(),
            x_min: x_min.into(),
            x_max: x_max.into(),
            y_min: y_min.into(),
            y_max: y_max.into(),
        }
    }

    /// Converts a raw device coordinate into millimeters.
    pub fn to_mm(&self, raw: Vec2<RawAxisValue>) -> Vec2<Millimeters> {
        Vec2::new(raw.x / self.x_res.as_f64(), raw.y / self.y_res.as_f64())
    }

    /// Physical width of the touch surface in millimeters.
    pub fn width_mm(&self) -> Millimeters {
        if self.x_res.as_f64() > 0.0 {
            Millimeters::new((self.x_max - self.x_min) / self.x_res)
        } else {
            Millimeters::new(0.0)
        }
    }

    /// Physical height of the touch surface in millimeters.
    pub fn height_mm(&self) -> Millimeters {
        if self.y_res.as_f64() > 0.0 {
            Millimeters::new((self.y_max - self.y_min) / self.y_res)
        } else {
            Millimeters::new(0.0)
        }
    }

    /// The whole usable surface as a [`crate::Region`] in millimeters -
    /// the bounds edge-ghost and zone filtering refer to.
    pub fn surface_region(&self) -> Region {
        Region {
            x_min: Millimeters::new(self.x_min.as_f64() / self.x_res.as_f64()),
            y_min: Millimeters::new(self.y_min.as_f64() / self.y_res.as_f64()),
            x_max: Millimeters::new(self.x_max.as_f64() / self.x_res.as_f64()),
            y_max: Millimeters::new(self.y_max.as_f64() / self.y_res.as_f64()),
        }
    }

    /// Maps a millimeter coordinate onto screen pixels.
    ///
    /// `screen_w`/`screen_h` are the pixel dimensions of the drawing area.
    /// Returns `(0.0, 0.0)` when no bounds/resolution are available.
    pub fn mm_to_screen(
        &self,
        pos: Vec2<Millimeters>,
        screen_w: Pixels,
        screen_h: Pixels,
    ) -> Vec2<Pixels> {
        let w_mm = self.width_mm().as_f64();
        let h_mm = self.height_mm().as_f64();
        if w_mm <= 0.0 || h_mm <= 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        let x_mm = pos.x - self.x_min / self.x_res;
        let y_mm = pos.y - self.y_min / self.y_res;
        Vec2::new(
            (x_mm / w_mm) * screen_w.as_f64(),
            (y_mm / h_mm) * screen_h.as_f64(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Calibration;
    use crate::RawAxisValue;
    use crate::Vec2;

    #[test]
    fn raw_coordinate_converts_to_mm() {
        let cal = Calibration::new(4.0, 2.0);
        let mm = cal.to_mm(Vec2::<RawAxisValue>::new(40.0, 20.0));
        assert!((mm.x - 10.0).abs() < 1e-9);
        assert!((mm.y - 10.0).abs() < 1e-9);
    }

    #[test]
    fn bounds_yield_physical_dimensions() {
        let cal = Calibration::with_bounds(10.0, 10.0, 0.0, 2000.0, 0.0, 1000.0);
        assert!((cal.width_mm().as_f64() - 200.0).abs() < 1e-9);
        assert!((cal.height_mm().as_f64() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn missing_resolution_yields_zero_dimensions() {
        let cal = Calibration::new(0.0, 0.0);
        assert_eq!(cal.width_mm().as_f64(), 0.0);
        assert_eq!(cal.height_mm().as_f64(), 0.0);
    }
}
