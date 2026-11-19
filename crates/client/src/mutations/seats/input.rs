//! Wire input objects for the seat mutations - `AddSeatInput` and
//! `UpdateSeatInput` as cynic DTOs of the schema's input objects.
//!
//! `Seat`/`SeatPose`/`TableSide` live in `smearor-commons-layout` and
//! cannot carry cynic derives (the orphan rule blocks `cynic::Enum`
//! on the external `TableSide`), so the nested inputs are wire
//! shapes converted from the shared types.

use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::RotationRad;
use smearor_commons_layout::Seat;
use smearor_commons_layout::SeatPose;
use smearor_commons_layout::TableSide;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;

/// `TableSide` wire enum - the schema enum values mirrored so cynic
/// can serialize the organizational side inside `SeatInput`.
#[derive(cynic::Enum, Clone, Copy)]
#[cynic(
    graphql_type = "TableSide",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub(crate) enum TableSideInput {
    /// Bottom edge.
    Bottom,
    /// Left edge.
    Left,
    /// Top edge.
    Top,
    /// Right edge.
    Right,
}

impl From<TableSide> for TableSideInput {
    fn from(side: TableSide) -> Self {
        match side {
            TableSide::Bottom => Self::Bottom,
            TableSide::Left => Self::Left,
            TableSide::Top => Self::Top,
            TableSide::Right => Self::Right,
        }
    }
}

/// `SeatInput` wire shape - organizational seat inside `SeatPoseInput`.
#[derive(cynic::InputObject)]
#[cynic(
    graphql_type = "SeatInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct SeatInput {
    /// The table edge the seat faces.
    side: TableSideInput,
    /// Position along the edge axis - `None` leaves distribution to
    /// the strategy.
    offset: Option<Fraction>,
}

impl From<Seat> for SeatInput {
    fn from(seat: Seat) -> Self {
        Self {
            side: seat.side.into(),
            offset: seat.offset,
        }
    }
}

/// `SeatPoseInput` wire shape - `seat` plus free radial orientation.
#[derive(cynic::InputObject)]
#[cynic(
    graphql_type = "SeatPoseInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct SeatPoseInput {
    /// Organizational alignment.
    seat: SeatInput,
    /// Radial orientation in radians.
    orientation: RotationRad,
}

impl From<SeatPose> for SeatPoseInput {
    fn from(pose: SeatPose) -> Self {
        Self {
            seat: pose.seat.into(),
            orientation: pose.orientation,
        }
    }
}

/// `AddSeatInput` wire shape - session, name, optional marker and pose.
#[derive(cynic::InputObject)]
#[cynic(
    graphql_type = "AddSeatInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct AddSeatInput {
    /// Session receiving the seat.
    pub(crate) session: SeatSessionName,
    /// Seat name - unique within the session.
    pub(crate) name: SeatName,
    /// Marker profile anchoring the seat - `None` = virtual.
    pub(crate) marker: Option<PatternId>,
    /// Initial pose - placeholder for marker-backed, applied for
    /// virtual seats.
    pub(crate) pose: Option<SeatPoseInput>,
}

/// `UpdateSeatInput` wire shape - `None` fields keep their value.
#[derive(cynic::InputObject, Default)]
#[cynic(
    graphql_type = "UpdateSeatInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct UpdateSeatInput {
    /// New seat name.
    pub(crate) name: Option<SeatName>,
    /// Rebinds the marker profile.
    pub(crate) marker: Option<PatternId>,
    /// Clears the marker binding - the seat turns virtual.
    pub(crate) clear_marker: Option<bool>,
    /// New pose.
    pub(crate) pose: Option<SeatPoseInput>,
}
