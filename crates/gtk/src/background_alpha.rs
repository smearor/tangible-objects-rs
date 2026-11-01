use std::fmt;

/// Canvas background opacity (0.0-1.0) - the alpha the cairo fill
/// applies, not a window-opacity hint: touches, figures and UI chrome
/// stay opaque at any value.
///
/// Newtype so the percent↔fraction conversion and the preset cycle
/// stay on the type instead of leaking into the CLI parse and the
/// control-bar callback.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BackgroundAlpha(f64);

/// Opacity presets the toggle cycles (GUI.md §1) - touch-friendly
/// steps from opaque down to fully transparent (only the content
/// stays: touches, figures, labels).
const PRESETS: [BackgroundAlpha; 5] = [
    BackgroundAlpha(1.0),
    BackgroundAlpha(0.75),
    BackgroundAlpha(0.5),
    BackgroundAlpha(0.25),
    BackgroundAlpha(0.0),
];

impl BackgroundAlpha {
    /// Clamps the fraction into the valid range.
    pub fn new(alpha: f64) -> Self {
        Self(alpha.clamp(0.0, 1.0))
    }

    /// Builds the alpha from a percent value (the CLI spelling).
    pub fn from_percent(percent: u8) -> Self {
        Self(f64::from(percent.min(100)) / 100.0)
    }

    /// Fraction form for `cairo::set_source_rgba`.
    pub fn as_f64(self) -> f64 {
        self.0
    }

    /// Percent form for labels and the CLI.
    pub fn percent(self) -> u8 {
        (self.0 * 100.0).round() as u8
    }

    /// The preset following this alpha in the cycle - wraps to opaque.
    pub fn next(self) -> Self {
        let index = PRESETS
            .iter()
            .position(|preset| (preset.0 - self.0).abs() < 0.01)
            .unwrap_or(0);
        PRESETS[(index + 1) % PRESETS.len()]
    }

    /// The `.surface-bg` CSS rule for the drawing area - GTK's own
    /// background layer blends rgba against the transparent window,
    /// which a cairo fill cannot (it would sit *above* the widget
    /// background and cover it opaquely). RGB is the dark canvas tint;
    /// the alpha channel is this value.
    pub fn surface_bg_css(self) -> String {
        format!(
            ".surface-bg {{ background-color: rgba(13, 13, 20, {}); }}",
            self.0
        )
    }
}

impl Default for BackgroundAlpha {
    /// Opaque by default.
    fn default() -> Self {
        Self(1.0)
    }
}

impl fmt::Display for BackgroundAlpha {
    /// Percent form - "75%".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percent())
    }
}

#[cfg(test)]
mod tests {
    use super::BackgroundAlpha;

    #[test]
    fn percent_roundtrips_through_fraction() {
        let alpha = BackgroundAlpha::from_percent(75);
        assert!((alpha.as_f64() - 0.75).abs() < 1e-9);
        assert_eq!(alpha.percent(), 75);
    }

    #[test]
    fn construction_clamps_into_range() {
        assert_eq!(BackgroundAlpha::new(1.5).as_f64(), 1.0);
        assert_eq!(BackgroundAlpha::new(-0.5).as_f64(), 0.0);
        assert_eq!(BackgroundAlpha::from_percent(255).as_f64(), 1.0);
    }

    #[test]
    fn next_cycles_presets_and_wraps() {
        assert_eq!(BackgroundAlpha::default().next().as_f64(), 0.75);
        assert_eq!(BackgroundAlpha(0.75).next().as_f64(), 0.5);
        assert_eq!(BackgroundAlpha(0.5).next().as_f64(), 0.25);
        assert_eq!(BackgroundAlpha(0.25).next().as_f64(), 0.0);
        assert_eq!(BackgroundAlpha(0.0).next().as_f64(), 1.0);
    }

    #[test]
    fn next_from_off_preset_starts_at_opaque() {
        assert_eq!(BackgroundAlpha(0.9).next().as_f64(), 0.75);
    }
}
