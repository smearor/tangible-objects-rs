//! `UnregisterPatternVariables` - the variables of
//! `unregisterPattern`.

use cynic::QueryVariables;
use tangible_objects_model::PatternId;

/// Variables of `unregisterPattern`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterPatternVariables {
    /// Registered pattern to drop.
    pub id: PatternId,
}
