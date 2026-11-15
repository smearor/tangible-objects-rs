use serde::Deserialize;
use serde::Serialize;

/// Integer rectangle extents `(x, y, width, height)` in pixels.
///
/// Wire shape for the bounds of a rasterized cairo [`Region`]: the
/// input-region diagnostics report every card footprint and the
/// unioned region as plain pixel extents so interfaces can compare
/// where the region places a card against where it renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RectExtents {
    /// Left edge in pixels.
    pub x: i32,
    /// Top edge in pixels.
    pub y: i32,
    /// Horizontal extent in pixels.
    pub width: i32,
    /// Vertical extent in pixels.
    pub height: i32,
}
