//! `PatternStyle` - a pattern's declared render appearance
//! (`ProfileFile.color`/`ProfileFile.shape`) mirrored into
//! [`SharedState`] for the draw loop.

use tangible_objects_model::FigureColor;
use tangible_objects_model::PatternId;
use tangible_objects_model::PhysicalShape;

/// The render-relevant profile declarations of one pattern.
#[derive(Debug, Clone)]
pub struct PatternStyle {
    /// Pattern the style belongs to.
    pub pattern_id: PatternId,
    /// Declared render color - `None` falls back to the hash palette.
    pub color: Option<FigureColor>,
    /// Declared physical outline - `None` renders contact visuals
    /// only, whatever the render mode.
    pub shape: Option<PhysicalShape>,
}
