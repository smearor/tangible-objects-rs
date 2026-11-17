use crate::PatternId;
use crate::PatternName;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `releaseProfile` applied - the profile returned to owner-less
/// state; carries the pattern's ID for symmetric handling with
/// [`crate::ProfileClaimed`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "ProfileReleased",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct ProfileReleased {
    /// ID of the released pattern.
    pub pattern_id: PatternId,
    /// Name of the released profile.
    pub name: PatternName,
}
