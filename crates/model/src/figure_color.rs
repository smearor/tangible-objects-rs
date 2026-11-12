//! `FigureColor` - the profile-declared render color of a tangible
//! (`color = "#rrggbb"` in the profile TOML).

use smearor_commons_color::Color;
use smearor_commons_color::ParseColorError;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// An RGB color for figure rendering, channels in `0.0..=1.0` -
/// a [`Color<f64>`] carrying the profile domain's `#rrggbb`
/// (lowercase) wire format.
///
/// Serializes as a `#rrggbb` hex string (a `#rgb` short form parses
/// too, each nibble duplicated).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FigureColor(pub Color<f64>);

impl FigureColor {
    /// Creates a color from `0.0..=1.0` channels.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Color::<f64>::new(r, g, b))
    }

    /// The channels as `(r, g, b)` tuple for cairo `set_source_*`.
    pub fn rgb(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

/// Direct field access (`color.r`) and shared [`Color<f64>`]
/// operations (`luminance`, `lerp`, `lighten`, `clamp`).
impl Deref for FigureColor {
    type Target = Color<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A shared [`Color<f64>`] as a profile figure color.
impl From<Color<f64>> for FigureColor {
    fn from(color: Color<f64>) -> Self {
        Self(color)
    }
}

/// The wrapped [`Color<f64>`].
impl From<FigureColor> for Color<f64> {
    fn from(color: FigureColor) -> Self {
        color.0
    }
}

impl Display for FigureColor {
    /// The `#rrggbb` spelling of the profile wire format, delegated
    /// to the wrapped color.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for FigureColor {
    type Err = ParseColorError;

    /// Parses `#rgb`/`#rrggbb` (the commons parser also accepts
    /// alpha forms and a missing `#`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl Serialize for FigureColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FigureColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::FigureColor;

    #[test]
    fn hex_roundtrip() {
        let color: FigureColor = "#40a0ff".parse().unwrap();
        assert!((color.r - 0x40 as f64 / 255.0).abs() < 1e-9);
        assert_eq!(color.to_string(), "#40a0ff");
    }

    #[test]
    fn short_hex_expands_nibbles() {
        let color: FigureColor = "#f80".parse().unwrap();
        assert_eq!(color.to_string(), "#ff8800");
    }
}
