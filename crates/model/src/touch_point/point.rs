use super::TouchPointSlot;
use super::TrackingId;
use crate::Millimeters;
use crate::Vec2;
#[cfg(feature = "cynic")]
use crate::schema;
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

#[cfg(feature = "zbus")]
impl zvariant::Type for TouchPoint {
    /// Flat `(slot, tracking_id, x, y)` - matches the `#[serde(flatten)]`
    /// field order.
    const SIGNATURE: &'static zvariant::Signature = &zvariant::signature!("uidd");
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

#[cfg(feature = "cynic")]
impl cynic::QueryFragment for TouchPoint {
    type SchemaType = schema::TouchPoint;
    type VariablesFields = ();
    const TYPE: Option<&'static str> = Some("TouchPoint");

    /// Selects `slot`, `trackingId` and `pos { x y }` - the response
    /// still decodes through the hand-written `Deserialize`, the
    /// fields are selection markers only.
    fn query(
        builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>,
    ) {
        let mut builder = builder;
        let _ =
            builder.select_field::<schema::__fields::TouchPoint::slot, schema::TouchPointSlot>();
        let _ =
            builder.select_field::<schema::__fields::TouchPoint::trackingId, schema::TrackingId>();
        let mut pos = builder.select_field::<schema::__fields::TouchPoint::pos, schema::Point>();
        let mut point = pos.select_children::<()>();
        let _ = point.select_field::<schema::__fields::Point::x, f64>();
        let _ = point.select_field::<schema::__fields::Point::y, f64>();
    }
}
