use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::Rotation;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::Vec2;

use crate::CardId;

/// One floating card's contribution to the touch-passthrough input
/// region, as last computed by the GUI's region sync.
///
/// Diagnostics only: comparing `position`/`rotation`/`scale` with the
/// rasterized `footprint` bounds shows whether the surface input
/// region actually covers the card where it renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardFootprint {
    /// Stable card identifier.
    pub card_id: CardId,
    /// Card center in board-layer pixels.
    pub position: Vec2<Pixels>,
    /// Card rotation around its center.
    pub rotation: Rotation,
    /// Card scale factor.
    pub scale: ScaleFactor,
    /// Whether the card's transform stack is GTK-visible.
    pub visible: bool,
    /// Rasterized footprint bounds in board-layer pixels; `None`
    /// when the card could not be measured or rasterized - a missing
    /// footprint makes the card completely input-dead.
    pub footprint: Option<Rect<Pixels, i32>>,
    /// Natural size the footprint was rasterized from (`measure_size`
    /// natural, unscaled). A degenerate value here rasterizes to a
    /// degenerate footprint even when the card renders fine.
    pub natural: Vec2<Pixels, i32>,
    /// The card widget's last GTK allocation - the ground truth of
    /// what is actually rendered on screen.
    pub allocated: Vec2<Pixels, i32>,
    /// Cairo status of the footprint region; `Some` when the region
    /// is in an error state and its contents are unusable.
    pub footprint_status: Option<String>,
    /// Rectangle count in the footprint region.
    pub footprint_rectangles: usize,
    /// First stored rectangle - where the rasterization actually
    /// landed when `extents` reads zero.
    pub first_rectangle: Option<Rect<Pixels, i32>>,
}
