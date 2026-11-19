use crate::TableTopSeat;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `addSeat` applied - carries the created seat with its assigned
/// `SeatId` so the client can address it for `updateSeat`/`removeSeat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SeatAdded",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SeatAdded {
    /// The added seat.
    pub seat: TableTopSeat,
}
