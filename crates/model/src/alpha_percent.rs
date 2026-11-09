//! `AlphaPercent` - an alpha channel spelled in whole percent
//! (0-100), the wire and domain form of alpha values (e.g.
//! `ControlCommand::SetBackgroundAlpha`); renderers convert to their
//! native form.

use std::fmt;

use serde::Deserialize;
use serde::Serialize;

/// Alpha/opacity spelled in percent, clamped to 0-100 - 100 is fully
/// opaque, 0 fully transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AlphaPercent(pub u8);

impl AlphaPercent {
    /// Fully opaque.
    pub const OPAQUE: Self = Self(100);

    /// Creates an alpha clamped into the valid range (0-100).
    pub const fn new(percent: u8) -> Self {
        Self(if percent > 100 { 100 } else { percent })
    }

    /// The raw percent value.
    pub const fn percent(self) -> u8 {
        self.0
    }

    /// Fraction form, 0.0-1.0, for renderers (e.g. cairo alpha).
    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

impl Default for AlphaPercent {
    /// Fully opaque - the alpha a canvas has without an explicit
    /// `--background-alpha` setting.
    fn default() -> Self {
        Self::OPAQUE
    }
}

impl From<u8> for AlphaPercent {
    /// Clamps the percent into the valid range.
    fn from(percent: u8) -> Self {
        Self::new(percent)
    }
}

impl From<AlphaPercent> for u8 {
    fn from(alpha: AlphaPercent) -> Self {
        alpha.0
    }
}

impl fmt::Display for AlphaPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::AlphaPercent;

    #[test]
    fn new_clamps_above_hundred() {
        assert_eq!(AlphaPercent::new(150).percent(), 100);
        assert_eq!(AlphaPercent::new(0).percent(), 0);
    }

    #[test]
    fn default_is_opaque() {
        assert_eq!(AlphaPercent::default(), AlphaPercent::OPAQUE);
    }
}
