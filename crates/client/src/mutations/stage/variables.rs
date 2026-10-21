//! `StageVariables` - shared variables of the stage mutations.

use cynic::QueryVariables;
use tangible_objects_model::StageName;

/// Variables of `registerStage`/`unregisterStage` - the stage `name`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct StageVariables {
    /// Stage name.
    pub name: StageName,
}
