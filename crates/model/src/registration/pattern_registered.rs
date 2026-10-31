use crate::PatternId;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `registerPattern` applied - carries the assigned pattern ID so the
/// client learns the registration's identity directly instead of
/// re-listing profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "PatternRegistered",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct PatternRegistered {
    /// Pattern ID assigned to the registered profile.
    pub pattern_id: PatternId,
}
