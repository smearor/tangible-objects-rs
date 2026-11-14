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
}
