//! `FigureColor` - the profile-declared render color of a tangible
//! (`color = "#rrggbb"` in the profile TOML).

use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// An RGB color for figure rendering, channels in `0.0..=1.0`.
///
/// Serializes as a `#rrggbb` hex string (a `#rgb` short form parses
/// too, each nibble duplicated).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FigureColor {
    /// Red channel.
    pub r: f64,
    /// Green channel.
    pub g: f64,
    /// Blue channel.
    pub b: f64,
}

impl FigureColor {
    /// Creates a color from `0.0..=1.0` channels.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// The channels as `(r, g, b)` tuple for cairo `set_source_*`.
    pub fn rgb(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

impl Display for FigureColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let byte = |channel: f64| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
        write!(
            f,
            "#{:02x}{:02x}{:02x}",
            byte(self.r),
            byte(self.g),
            byte(self.b)
        )
    }
}

impl FromStr for FigureColor {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid figure color '{s}' - expected #rgb or #rrggbb"),
            )
        };
        let hex = s.strip_prefix('#').ok_or_else(invalid)?;
        let byte = |from: &str| u8::from_str_radix(from, 16).map_err(|_| invalid());
        let (r, g, b) = match hex.len() {
            3 => (
                byte(&hex[0..1].repeat(2))?,
                byte(&hex[1..2].repeat(2))?,
                byte(&hex[2..3].repeat(2))?,
            ),
            6 => (byte(&hex[0..2])?, byte(&hex[2..4])?, byte(&hex[4..6])?),
            _ => return Err(invalid()),
        };
        Ok(Self::new(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
        ))
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
