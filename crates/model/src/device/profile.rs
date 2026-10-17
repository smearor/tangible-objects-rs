use serde::Deserialize;
use serde::Serialize;

use crate::Calibration;
use crate::EventCode;
use crate::EventType;
use crate::Millimeters;
use crate::RawAxisValue;
use crate::UnitsPerMm;

/// Complete profile of a touchscreen device - everything needed to convert
/// raw `.evtest` recordings between devices (TRAINING.md device profiles).
///
/// The TOML shape mirrors the `evtest` capability header: axis bounds and
/// resolutions come straight from the `ABS_MT_POSITION_X/Y` blocks,
/// `abs_codes`/`key_codes` list every event code the device emits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    /// Human-readable device name - the `Input device name:` header line.
    pub name: String,
    /// evdev vendor ID for auto-matching (`vendor` in the evtest header).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<u16>,
    /// evdev product ID for auto-matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product: Option<u16>,
    /// Device units per millimeter on X (`Resolution` under `ABS_MT_POSITION_X`).
    pub x_res: UnitsPerMm,
    /// Device units per millimeter on Y (`Resolution` under `ABS_MT_POSITION_Y`).
    pub y_res: UnitsPerMm,
    /// Minimum raw X value.
    pub x_min: RawAxisValue,
    /// Maximum raw X value.
    pub x_max: RawAxisValue,
    /// Minimum raw Y value.
    pub y_min: RawAxisValue,
    /// Maximum raw Y value.
    pub y_max: RawAxisValue,
    /// `EV_ABS` codes the device emits (e.g. `53`/`54`/`57`, plus legacy
    /// `0`/`1` where present). Converting to this device drops any source
    /// event whose code is not listed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub abs_codes: Vec<u16>,
    /// `EV_KEY` codes the device emits (e.g. `330` = `BTN_TOUCH`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_codes: Vec<u16>,
    /// Typical report rate in Hz - informational; conversion keeps the
    /// source timestamps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_rate_hz: Option<u16>,
}

impl DeviceProfile {
    /// The affine axis map as a [`Calibration`].
    pub fn calibration(&self) -> Calibration {
        Calibration::with_bounds(
            self.x_res, self.y_res, self.x_min, self.x_max, self.y_min, self.y_max,
        )
    }

    /// Whether the device emits `(event_type, code)` - conversion drops
    /// events outside the profile's set.
    pub fn supports(&self, event_type: EventType, code: EventCode) -> bool {
        match event_type {
            EventType::Syn => true, // SYN_REPORT is a delimiter, not a capability
            EventType::Abs => self.abs_codes.contains(&code.as_u16()),
            EventType::Key => self.key_codes.contains(&code.as_u16()),
            EventType::Other(_) => false,
        }
    }

    /// Converts a millimeter position to this device's raw axis value -
    /// the inverse of [`Calibration::to_mm`], clamped to `[min, max]`.
    ///
    /// Returns the clamped raw value and whether clamping occurred (the
    /// source geometry exceeded this device's bounds).
    pub fn to_raw(&self, position: Millimeters, axis_x: bool) -> (RawAxisValue, bool) {
        let (res, min, max) = if axis_x {
            (self.x_res, self.x_min, self.x_max)
        } else {
            (self.y_res, self.y_min, self.y_max)
        };
        let raw = position.as_f64() * res.as_f64() + min.as_f64();
        let clamped = raw.clamp(min.as_f64(), max.as_f64());
        (
            RawAxisValue::new(clamped),
            (clamped - raw).abs() > f64::EPSILON,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceProfile;
    use crate::EventCode;
    use crate::EventType;
    use crate::Millimeters;

    fn profile() -> DeviceProfile {
        DeviceProfile {
            name: "test".to_string(),
            vendor: None,
            product: None,
            x_res: 8.0.into(),
            y_res: 8.0.into(),
            x_min: 0.0.into(),
            x_max: 4000.0.into(),
            y_min: 0.0.into(),
            y_max: 4000.0.into(),
            abs_codes: vec![53, 54, 57],
            key_codes: vec![330],
            report_rate_hz: Some(120),
        }
    }

    #[test]
    fn to_raw_inverts_to_mm() {
        let profile = profile();
        let calibration = profile.calibration();
        // 250 mm on the source axis → 2000 raw on this device (res 8).
        let (raw, clamped) = profile.to_raw(Millimeters::new(250.0), true);
        assert_eq!(raw.as_f64(), 2000.0);
        assert!(!clamped);
        // Round-trip through to_mm.
        let back = calibration.to_mm(crate::Vec2::new(raw.as_f64(), raw.as_f64()));
        assert!((back.x - 250.0).abs() < 0.01);
    }

    #[test]
    fn to_raw_clamps_out_of_bounds() {
        let profile = profile();
        // 600 mm exceeds max (4000 raw = 500 mm) → clamped to the bound.
        let (raw, clamped) = profile.to_raw(Millimeters::new(600.0), true);
        assert_eq!(raw.as_f64(), 4000.0);
        assert!(clamped);
    }

    #[test]
    fn supports_only_listed_codes() {
        let profile = profile();
        assert!(profile.supports(EventType::Abs, EventCode::new(53)));
        assert!(!profile.supports(EventType::Abs, EventCode::new(0)));
        assert!(profile.supports(EventType::Key, EventCode::new(330)));
        assert!(profile.supports(EventType::Syn, EventCode::new(0)));
    }
}
