//! `TrainProfileVariables` - the variables of `trainProfile`.

use cynic::QueryVariables;
use tangible_objects_model::PatternName;

/// Variables of `trainProfile` - the figure name.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct TrainProfileVariables {
    /// Profile name to train.
    pub name: PatternName,
}
