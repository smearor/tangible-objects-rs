use crate::PatternId;
use crate::PatternName;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `claimProfile` applied - carries the claimed pattern's ID so the
/// client can adopt it (track/unregister) directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "ProfileClaimed",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct ProfileClaimed {
    /// ID of the claimed pattern.
    pub pattern_id: PatternId,
    /// Name of the claimed profile.
    pub name: PatternName,
}
