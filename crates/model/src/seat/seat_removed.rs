use crate::TableTopSeat;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `removeSeat` applied - carries the removed seat's last state so
/// the client can confirm what left the roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SeatRemoved",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SeatRemoved {
    /// The removed seat.
    pub seat: TableTopSeat,
}
