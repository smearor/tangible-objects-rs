#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// Whether a [`crate::TableTopSeat`] is currently anchored by a
/// tracked marker tangible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(not(feature = "cynic"), serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
pub enum SeatMarkerState {
    /// Marker-backed seat bound to a tracked marker tangible - `pose`
    /// follows the tangible every frame.
    Anchored,
    /// Marker-backed seat whose marker is not currently tracked -
    /// `pose` retains the last anchored pose (absent hysteresis) until
    /// the marker returns or the application removes the seat.
    Absent,
    /// Seat without a marker - placed explicitly by the application.
    Virtual,
}
