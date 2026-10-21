//! `UpdateProfileVariables` - the variables of `updateProfile`.

use tangible_objects_model::PatternId;

use crate::mutations::pattern::input::RegisterPatternInput;
use cynic::QueryVariables;

/// Variables of `updateProfile`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UpdateProfileVariables {
    /// Registered pattern to replace.
    pub id: PatternId,
    /// The new pattern geometry.
    pub input: RegisterPatternInput,
}
