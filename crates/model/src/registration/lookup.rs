use crate::ProfileLookupMatch;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `profileLookup` result - how a candidate profile compares against
/// the registered patterns.
///
/// Runs the same fingerprint collision check `registerPattern`
/// rejects on: `matching` reports the first registered pattern whose
/// fingerprint the candidate fits within tolerance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "ProfileLookup",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct ProfileLookup {
    /// The colliding registered pattern; `None` means
    /// `registerPattern` with the same input would not hit a
    /// `FINGERPRINT_COLLISION` rejection.
    pub matching: Option<ProfileLookupMatch>,
}
