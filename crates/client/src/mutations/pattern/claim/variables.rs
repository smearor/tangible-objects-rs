//! `ClaimProfileVariables` - the variables of `claimProfile`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;

/// Variables of `claimProfile`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ClaimProfileVariables {
    /// ID of the profile to claim - `profileLookup`'s match carries
    /// it; profile names are not unique.
    pub id: PatternId,
    /// Claiming application.
    pub owner: Owner,
}
