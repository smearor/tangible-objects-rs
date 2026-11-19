use crate::TableTopSeat;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `updateSeat` applied - carries the seat's state after the
/// mutation so the client sees the effective name/marker/pose in one
/// reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SeatUpdated",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SeatUpdated {
    /// The seat after the update.
    pub seat: TableTopSeat,
}
