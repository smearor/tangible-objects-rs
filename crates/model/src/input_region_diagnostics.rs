use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::Vec2;

use crate::CardFootprint;

/// The touch-passthrough input region as last applied to the GUI
/// surface - diagnostics for touches not reaching widgets they
/// visibly hit.
///
/// Emitted by the daemon-embedded GUI's region sync (~10 Hz while
/// `touch_passthrough` is on). Headless daemons and the disabled
/// state report `applied: false` with empty data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputRegionDiagnostics {
    /// Whether `set_input_region` actually ran on the last tick.
    pub applied: bool,
    /// Board-to-surface origin offset applied to every footprint
    /// before upload, in pixels.
    pub origin: Vec2<Pixels>,
    /// Bounds of the unioned region in surface pixels; `None` when
    /// the region is empty (whole surface input-dead).
    pub extents: Option<Rect<Pixels, i32>>,
    /// Rectangle count in the unioned region - complexity signal.
    pub rectangles: usize,
    /// Cairo status of the unioned region; `Some` when the region is
    /// in an error state (extents/contents then report garbage).
    pub status: Option<String>,
    /// First stored rectangle of the unioned region - where the
    /// rasterization actually landed when `extents` reads zero.
    pub first_rectangle: Option<Rect<Pixels, i32>>,
    /// Per-card footprints in board-layer coordinates.
    pub cards: Vec<CardFootprint>,
}
