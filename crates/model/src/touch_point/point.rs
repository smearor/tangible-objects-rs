use super::TouchPointSlot;
use super::TrackingId;
use crate::Millimeters;
use crate::Vec2;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

/// A single active multitouch contact tracked by its slot.
///
/// Serializes flat as `{slot, tracking_id, x, y}` via `#[serde(flatten)]` on
/// the position.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct TouchPoint {
    /// Multitouch slot number the contact is assigned to.
    pub slot: TouchPointSlot,
    /// Kernel-assigned tracking ID of the contact lifetime.
    pub tracking_id: TrackingId,
    /// Contact position (raw device units when uncalibrated).
    #[serde(flatten)]
    pub pos: Vec2<Millimeters>,
}

impl<'de> Deserialize<'de> for TouchPoint {
    /// Accepts both wire shapes: the GraphQL `SimpleObject` form
    /// (`pos {x,y}` + `trackingId`, camelCase keys) and the legacy flat
    /// serde form (`tracking_id` + `x`/`y` at top level).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Helper {
            slot: TouchPointSlot,
            #[serde(alias = "tracking_id")]
            tracking_id: TrackingId,
            pos: Option<Vec2<Millimeters>>,
            #[serde(default, alias = "x")]
            x: Option<f64>,
            #[serde(default, alias = "y")]
            y: Option<f64>,
        }
        let h = Helper::deserialize(deserializer)?;
        let pos = h
            .pos
            .unwrap_or_else(|| Vec2::new(h.x.unwrap_or_default(), h.y.unwrap_or_default()));
        Ok(Self {
            slot: h.slot,
            tracking_id: h.tracking_id,
            pos,
        })
    }
}
