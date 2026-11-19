use crate::SeatSession;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `claimSeatSession` applied - carries the claimed session (name,
/// owner, roster) so the client mirrors it without a follow-up query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SeatSessionClaimed",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SeatSessionClaimed {
    /// The claimed session - empty roster until `addSeat` calls fill it.
    pub session: SeatSession,
}
