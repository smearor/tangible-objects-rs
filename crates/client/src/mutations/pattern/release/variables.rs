//! `ReleaseProfileVariables` - the variables of `releaseProfile`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;

/// Variables of `releaseProfile`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ReleaseProfileVariables {
    /// ID of the profile to release - profile names are not unique.
    pub id: PatternId,
    /// Owner giving up its claim.
    pub owner: Owner,
    /// Deactivate the profile in the same operation - `None`/`false`
    /// keeps its current enabled state.
    pub disable: Option<bool>,
}
