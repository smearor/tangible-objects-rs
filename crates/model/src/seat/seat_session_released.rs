use crate::SeatSession;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `releaseSeatSession` applied - carries the released session with
/// the roster that went down with it, so the client can drop its
/// local seat state in one step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SeatSessionReleased",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SeatSessionReleased {
    /// The released session including its final roster.
    pub session: SeatSession,
}
