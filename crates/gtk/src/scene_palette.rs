//! The semantic color palette for the cairo scene overlays - the one
//! place to restyle the daemon GUI. Slots are enum-typed via
//! [`palette_definition!`], so every lookup is compile-checked and
//! [`Palette::is_complete`] can assert that all slots are filled.

use std::sync::LazyLock;

use smearor_commons_color::Color;
use smearor_commons_color::Palette;
use smearor_commons_color::palette_definition;

palette_definition! {
    /// Semantic color slots for the scene overlays.
    pub enum SceneColor {
        /// Device error banner text.
        Error => "error",
        /// Interface-pushed message text.
        Message => "message",
        /// "No calibration" / empty-state notices.
        Notice => "notice",
        /// Millimeter grid lattice and its border.
        Grid => "grid",
        /// Dead-zone fill and outline.
        DeadZone => "dead-zone",
        /// Dead-zone name label.
        DeadZoneLabel => "dead-zone-label",
        /// Edge-margin overlay.
        EdgeMargin => "edge-margin",
        /// Field fill and outline.
        Field => "field",
        /// Field name label.
        FieldLabel => "field-label",
        /// Seat marker fill/outline for seats without a profile color
        /// (virtual or profile-less seats).
        Seat => "seat",
        /// Seat name label.
        SeatLabel => "seat-label",
        /// Ruler strip background.
        RulerBg => "ruler-bg",
        /// Ruler ticks and mm labels.
        Ruler => "ruler",
        /// Position labels and leader lines.
        Label => "label",
        /// "No touches" training hint.
        EmptyState => "empty-state",
        /// Pairwise connection lines in the training graph.
        TouchLine => "touch-line",
        /// Stable (confirmed) touch point.
        TouchStable => "touch-stable",
        /// Unstable touch point.
        TouchUnstable => "touch-unstable",
        /// Coordinate text under touch points.
        CoordText => "coord-text",
        /// Unmatched touch points.
        TouchDim => "touch-dim",
        /// Primary white - matched points, names, outlines.
        White => "white"
    }
}

/// The scene palette - all slots filled at first use.
pub static SCENE: LazyLock<Palette<SceneColor, f64>> = LazyLock::new(|| {
    Palette::<SceneColor, f64>::builder()
        .color(SceneColor::Error, Color::<f64>::new(1.0, 0.3, 0.3))
        .color(SceneColor::Message, Color::<f64>::new(1.0, 0.9, 0.2))
        .color(SceneColor::Notice, Color::<f64>::new(0.8, 0.8, 0.8))
        .color(SceneColor::Grid, Color::<f64>::new(0.8, 0.8, 0.8))
        .color(SceneColor::DeadZone, Color::<f64>::new(1.0, 0.3, 0.25))
        .color(SceneColor::DeadZoneLabel, Color::<f64>::new(1.0, 0.55, 0.5))
        .color(SceneColor::EdgeMargin, Color::<f64>::new(1.0, 0.35, 0.2))
        .color(SceneColor::Field, Color::<f64>::new(0.2, 0.8, 1.0))
        .color(SceneColor::FieldLabel, Color::<f64>::new(0.55, 0.9, 1.0))
        .color(SceneColor::Seat, Color::<f64>::new(0.9, 0.7, 0.3))
        .color(SceneColor::SeatLabel, Color::<f64>::new(1.0, 0.85, 0.5))
        .color(SceneColor::RulerBg, Color::<f64>::new(0.04, 0.04, 0.06))
        .color(SceneColor::Ruler, Color::<f64>::new(0.9, 0.9, 0.9))
        .color(SceneColor::Label, Color::<f64>::new(0.85, 0.85, 0.85))
        .color(SceneColor::EmptyState, Color::<f64>::new(0.6, 0.6, 0.6))
        .color(SceneColor::TouchLine, Color::<f64>::new(0.4, 0.7, 1.0))
        .color(SceneColor::TouchStable, Color::<f64>::new(0.3, 0.9, 0.4))
        .color(SceneColor::TouchUnstable, Color::<f64>::new(0.5, 0.7, 1.0))
        .color(SceneColor::CoordText, Color::<f64>::new(0.7, 0.7, 0.7))
        .color(SceneColor::TouchDim, Color::<f64>::new(0.4, 0.4, 0.4))
        .color(SceneColor::White, Color::<f64>::WHITE)
        .build()
});

/// Per-figure accent colors - stable hash over the pattern ID.
pub const FIGURE_ACCENTS: [Color<f64>; 6] = [
    Color::<f64>::new(0.2, 0.6, 1.0),
    Color::<f64>::new(0.2, 0.9, 0.3),
    Color::<f64>::new(1.0, 0.5, 0.2),
    Color::<f64>::new(1.0, 0.3, 0.5),
    Color::<f64>::new(0.8, 0.3, 1.0),
    Color::<f64>::new(1.0, 0.9, 0.2),
];

impl SceneColor {
    /// The color registered for this slot - `Color::<f64>::BLACK` only if a
    /// slot was left unfilled (the completeness test prevents that).
    pub fn color(self) -> Color<f64> {
        SCENE.get_or(&self, Color::<f64>::BLACK)
    }
}

#[cfg(test)]
mod tests {
    use super::SCENE;

    #[test]
    fn scene_palette_is_complete() {
        assert!(SCENE.is_complete(), "missing slots: {:?}", SCENE.missing());
    }
}
