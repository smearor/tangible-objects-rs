use super::TouchPointSlot;
use crate::Millimeters;
use crate::Vec2;

/// A single active multitouch contact tracked by its slot.
#[derive(Debug, Clone)]
pub struct TouchPoint {
    /// Multitouch slot number the contact is assigned to.
    pub slot: TouchPointSlot,
    /// Kernel-assigned tracking ID of the contact lifetime.
    pub tracking_id: i32,
    /// Contact position in millimeters (or raw device units when no
    /// calibration is set).
    pub pos: Vec2<Millimeters>,
}
