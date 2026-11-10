//! Where the active surface calibration came from - shown on the
//! calibration card.

/// Origin of the calibration the engine runs with: the device's
/// firmware-reported axes or a device-profile override (the
/// `effective_calibration` hook applies the profile silently - the
/// card shows which won by comparing both).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationSource {
    /// Calibration reported by the input device itself.
    Device,
    /// Calibration from a device profile overriding the firmware
    /// values.
    Profile,
}
