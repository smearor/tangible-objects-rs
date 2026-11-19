use crate::Owner;
use crate::PatternId;
use crate::SeatId;
use crate::SeatMarkerState;
use crate::SeatName;
use crate::SeatSessionName;
use crate::TangibleId;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;
use smearor_commons_layout::SeatPose;

/// A physical place at the tabletop inside a [`crate::SeatSession`]
/// (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
///
/// Marker-backed seats (`marker: Some`) anchor to the tracked tangible
/// matching that profile - `pose` is re-derived from it every frame
/// and `tangible` carries the bound instance. Virtual seats
/// (`marker: None`) keep the application-provided pose. `state`
/// reports the anchor's health: an `Absent` seat retains its last
/// pose until the marker returns or the application removes the seat.
///
/// A seat is a *place*, not a person: who sits there - display name,
/// player color, rights - stays in the application's domain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TableTopSeat {
    /// Stable seat ID - unique across sessions, never reused.
    pub id: SeatId,
    /// Application-facing seat name, unique within its session.
    pub name: SeatName,
    /// Session this seat belongs to.
    pub session: SeatSessionName,
    /// Application owning the session - mirrors `session.owner` so the
    /// Seats card and event payloads carry it without a join.
    pub owner: Owner,
    /// Marker profile anchoring this seat - the figure profile whose
    /// tracked instances drive `pose`. `None` = virtual seat.
    pub marker: Option<PatternId>,
    /// Currently bound marker tangible instance - `None` while the
    /// marker is absent or for virtual seats.
    pub tangible: Option<TangibleId>,
    /// Organizational plus radial pose: table side, edge offset and
    /// free-angle orientation. Marker-derived while anchored,
    /// application-provided while virtual, last anchored pose while
    /// absent.
    pub pose: SeatPose,
    /// Marker anchor state.
    pub state: SeatMarkerState,
}

#[cfg(feature = "cynic")]
impl cynic::QueryFragment for TableTopSeat {
    type SchemaType = schema::TableTopSeat;
    type VariablesFields = ();
    const TYPE: Option<&'static str> = Some("TableTopSeat");

    /// Selects the eight schema fields - `pose` expands into its
    /// nested `seat { side offset }` and `orientation` leaves, decoded
    /// by `SeatPose`'s serde impl (the smearor type cannot carry a
    /// cynic derive).
    fn query(
        builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>,
    ) {
        let mut builder = builder;
        let _ = builder.select_field::<schema::__fields::TableTopSeat::id, schema::SeatId>();
        let _ = builder.select_field::<schema::__fields::TableTopSeat::name, schema::SeatName>();
        let _ = builder
            .select_field::<schema::__fields::TableTopSeat::session, schema::SeatSessionName>();
        let _ = builder.select_field::<schema::__fields::TableTopSeat::owner, schema::Owner>();
        let _ = builder
            .select_field::<schema::__fields::TableTopSeat::marker, Option<schema::PatternId>>();
        let _ = builder
            .select_field::<schema::__fields::TableTopSeat::tangible, Option<schema::TangibleId>>();
        let mut pose =
            builder.select_field::<schema::__fields::TableTopSeat::pose, schema::SeatPose>();
        let mut pose_fields = pose.select_children::<()>();
        let mut seat = pose_fields.select_field::<schema::__fields::SeatPose::seat, schema::Seat>();
        let mut seat_fields = seat.select_children::<()>();
        let _ = seat_fields.select_field::<schema::__fields::Seat::side, schema::TableSide>();
        let _ =
            seat_fields.select_field::<schema::__fields::Seat::offset, Option<schema::Fraction>>();
        let _ = pose_fields
            .select_field::<schema::__fields::SeatPose::orientation, schema::RotationRad>();
        let _ = builder
            .select_field::<schema::__fields::TableTopSeat::state, schema::SeatMarkerState>();
    }
}
