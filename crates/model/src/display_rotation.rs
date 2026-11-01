use std::fmt;
use std::str::FromStr;

use crate::Millimeters;
use crate::Vec2;

/// Quarter-turn rotation of the *display output* relative to the touch
/// panel's native orientation.
///
/// The evdev device always reports sensor coordinates in the panel's
/// physical landscape frame - rotating the picture (e.g. GNOME portrait
/// mode) does not rotate the input stream. `DisplayRotation` bridges the
/// two: applied to millimeter positions before the mm-to-pixel scale so
/// rendered contacts match what the user sees on the rotated screen.
///
/// The direction is **clockwise as seen by the viewer**: `Deg90` is the
/// panel's native landscape top edge displayed on the viewer's right.
/// When the desktop is rotated the other way, use `Deg270`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum DisplayRotation {
    /// Display orientation matches the panel's native orientation.
    #[default]
    Deg0,
    /// Display rotated 90° clockwise relative to the panel.
    Deg90,
    /// Display upside down relative to the panel.
    Deg180,
    /// Display rotated 270° clockwise relative to the panel.
    Deg270,
}

impl DisplayRotation {
    /// Whether the quarter turn swaps the surface's width and height.
    pub fn swaps_axes(self) -> bool {
        matches!(self, Self::Deg90 | Self::Deg270)
    }

    /// Rotates a millimeter position inside a `w_mm × h_mm` surface into
    /// the displayed orientation. The surface bounds swap when the
    /// quarter turn swaps axes - the caller uses the returned position
    /// against `(h_mm, w_mm)` in that case.
    pub fn apply(
        self,
        pos: Vec2<Millimeters>,
        w_mm: Millimeters,
        h_mm: Millimeters,
    ) -> Vec2<Millimeters> {
        match self {
            Self::Deg0 => pos,
            Self::Deg90 => Vec2::new(pos.y, w_mm.as_f64() - pos.x),
            Self::Deg180 => Vec2::new(w_mm.as_f64() - pos.x, h_mm.as_f64() - pos.y),
            Self::Deg270 => Vec2::new(h_mm.as_f64() - pos.y, pos.x),
        }
    }
}

impl fmt::Display for DisplayRotation {
    /// The degrees spelling accepted by [`FromStr`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let deg = match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        };
        write!(f, "{deg}")
    }
}

impl FromStr for DisplayRotation {
    type Err = String;

    /// Accepts the quarter-turn values `0`, `90`, `180` or `270`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Deg0),
            "90" => Ok(Self::Deg90),
            "180" => Ok(Self::Deg180),
            "270" => Ok(Self::Deg270),
            _ => Err(format!(
                "invalid display rotation '{s}' - use 0, 90, 180 or 270"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayRotation;
    use crate::Millimeters;
    use crate::Vec2;

    #[test]
    fn quarter_turns_rotate_inside_bounds() {
        let w = Millimeters::new(400.0);
        let h = Millimeters::new(300.0);
        let pos = Vec2::<Millimeters>::new(100.0, 50.0);

        let deg90 = DisplayRotation::Deg90.apply(pos, w, h);
        assert!((deg90.x - 50.0).abs() < 1e-9, "x becomes y");
        assert!((deg90.y - 300.0).abs() < 1e-9, "y becomes w - x");

        let deg180 = DisplayRotation::Deg180.apply(pos, w, h);
        assert!((deg180.x - 300.0).abs() < 1e-9);
        assert!((deg180.y - 250.0).abs() < 1e-9);

        let deg270 = DisplayRotation::Deg270.apply(pos, w, h);
        assert!((deg270.x - 250.0).abs() < 1e-9, "x becomes h - y");
        assert!((deg270.y - 100.0).abs() < 1e-9, "y becomes x");
    }

    #[test]
    fn quarter_turns_swap_axes() {
        assert!(DisplayRotation::Deg90.swaps_axes());
        assert!(DisplayRotation::Deg270.swaps_axes());
        assert!(!DisplayRotation::Deg0.swaps_axes());
        assert!(!DisplayRotation::Deg180.swaps_axes());
    }

    #[test]
    fn display_round_trips_parsing() {
        for deg in ["0", "90", "180", "270"] {
            let rotation: DisplayRotation = deg.parse().expect("valid degree");
            assert_eq!(rotation.to_string(), deg);
        }
        assert!("45".parse::<DisplayRotation>().is_err());
    }
}
