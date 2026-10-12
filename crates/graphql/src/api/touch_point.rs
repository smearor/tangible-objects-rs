use async_graphql::SimpleObject;
use tangible_objects_model::TouchPoint as ModelTouchPoint;

use super::Point;

/// A single active multitouch contact.
#[derive(Debug, Clone, SimpleObject)]
pub struct TouchPoint {
    /// Kernel multitouch slot the contact is assigned to.
    pub slot: u32,
    /// Kernel-assigned tracking ID of the contact lifetime.
    pub tracking_id: i32,
    /// Contact position in millimeters.
    pub pos: Point,
}

impl From<&ModelTouchPoint> for TouchPoint {
    fn from(touch: &ModelTouchPoint) -> Self {
        Self {
            slot: touch.slot.as_u32(),
            tracking_id: touch.tracking_id.as_i32(),
            pos: touch.pos.into(),
        }
    }
}
