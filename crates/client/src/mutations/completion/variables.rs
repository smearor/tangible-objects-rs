//! `SetCompletionVariables` - the variables of `setCompletion`.

use crate::mutations::completion::input::CompletionInput;
use cynic::QueryVariables;

/// Variables of `setCompletion`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetCompletionVariables {
    /// The replacement config.
    pub config: CompletionInput,
}
