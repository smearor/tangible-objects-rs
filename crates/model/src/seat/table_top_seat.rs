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
    #[serde(with = "pose_serde")]
    #[cfg_attr(feature = "mcp", schemars(with = "SeatPose"))]
    pub pose: SeatPose,
    /// Marker anchor state.
    pub state: SeatMarkerState,
}

/// Serde `with` helpers for [`TableTopSeat::pose`]: the GraphQL wire
/// spells enum variants SCREAMING_SNAKE (`BOTTOM`) while `TableSide`'s
/// canonical serde form is lowercase (`bottom`). Decoding through a
/// wire mirror accepts both spellings; serialization keeps the
/// canonical lowercase form `SeatPose` already emits.
mod pose_serde {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use smearor_commons_geometry::Fraction;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_layout::Seat;
    use smearor_commons_layout::SeatPose;
    use smearor_commons_layout::TableSide;

    /// `Seat` wire mirror - `side` decodes through [`WireSide`].
    #[derive(Deserialize)]
    struct WireSeat {
        /// Table edge.
        side: WireSide,
        /// Position along the edge axis.
        offset: Option<Fraction>,
    }

    /// `SeatPose` wire mirror.
    #[derive(Deserialize)]
    struct WirePose {
        /// Organizational alignment.
        seat: WireSeat,
        /// Radial orientation.
        orientation: RotationRad,
    }

    /// `TableSide` accepting its GraphQL (`BOTTOM`) and canonical
    /// serde (`bottom`) spellings.
    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum WireSide {
        /// Bottom edge.
        #[serde(alias = "BOTTOM")]
        Bottom,
        /// Left edge.
        #[serde(alias = "LEFT")]
        Left,
        /// Top edge.
        #[serde(alias = "TOP")]
        Top,
        /// Right edge.
        #[serde(alias = "RIGHT")]
        Right,
    }

    impl From<WireSide> for TableSide {
        fn from(side: WireSide) -> Self {
            match side {
                WireSide::Bottom => Self::Bottom,
                WireSide::Left => Self::Left,
                WireSide::Top => Self::Top,
                WireSide::Right => Self::Right,
            }
        }
    }

    /// Serializes through `SeatPose`'s own impl - canonical lowercase.
    pub(super) fn serialize<S: Serializer>(
        pose: &SeatPose,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        pose.serialize(serializer)
    }

    /// Deserializes through the wire mirror - accepts `BOTTOM` and
    /// `bottom` spellings of `TableSide`.
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<SeatPose, D::Error> {
        let wire = WirePose::deserialize(deserializer)?;
        Ok(SeatPose::new(
            Seat {
                side: wire.seat.side.into(),
                offset: wire.seat.offset,
            },
            wire.orientation,
        ))
    }
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
