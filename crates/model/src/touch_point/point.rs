use super::TouchPointSlot;
use super::TrackingId;
use crate::Millimeters;
use crate::Vec2;
use serde::Deserialize;
use serde::Serialize;

/// A single active multitouch contact tracked by its slot.
///
/// Serializes flat as `{slot, tracking_id, x, y}` via `#[serde(flatten)]` on
/// the position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    /// Multitouch slot number the contact is assigned to.
    pub slot: TouchPointSlot,
    /// Kernel-assigned tracking ID of the contact lifetime.
    pub tracking_id: TrackingId,
    /// Contact position (raw device units when uncalibrated).
    #[serde(flatten)]
    pub pos: Vec2<Millimeters>,
}
