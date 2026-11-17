use crate::Owner;
use crate::PatternId;
use crate::PatternName;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// A registered pattern matching a `profileLookup` candidate's
/// fingerprint - the answer to "does the daemon already have this
/// profile, and is it the same one".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "ProfileLookupMatch",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct ProfileLookupMatch {
    /// ID of the matching registered pattern.
    pub pattern_id: PatternId,
    /// Name of the matching registered pattern - compare against the
    /// candidate's name; fingerprints alone do not prove name equality.
    pub name: PatternName,
    /// Owning application of the registered pattern; `None` for
    /// statically loaded or released profiles - `claimProfile`
    /// targets exactly this state.
    pub owner: Option<Owner>,
    /// Exact fingerprint equality: the registered pattern's sorted
    /// distance fingerprint equals the candidate's elementwise.
    /// `false` means the candidate merely collides within the
    /// registered pattern's tolerance - a different profile geometry.
    pub identical: bool,
}
