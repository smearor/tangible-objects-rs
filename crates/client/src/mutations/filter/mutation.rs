//! `SetFilterMode` - the cynic fragment for the `setFilterMode`
//! mutation.

use crate::mutations::filter::variables::SetFilterModeVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for the `setFilterMode` mutation - cynic
/// derives the document and checks every field and argument against
/// `schema.graphql` at compile time. The fields are selection
/// markers only - the response decodes through serde.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetFilterModeVariables"
)]
pub(crate) struct SetFilterMode {
    /// `setFilterMode` field - returns whether the switch applied.
    #[cynic(rename = "setFilterMode")]
    #[arguments(mode: $mode, stability: $stability, stage: $stage)]
    set_filter_mode: bool,
}

#[cfg(test)]
mod tests {
    use super::SetFilterMode;
    use crate::mutations::filter::variables::SetFilterModeVariables;
    use cynic::Operation;
    use tangible_objects_model::FilterMode;

    /// The cynic-derived document declares the schema's argument
    /// types and applies them to `setFilterMode`.
    #[test]
    fn set_filter_mode_document() {
        let operation =
            Operation::<SetFilterMode, SetFilterModeVariables>::mutation(SetFilterModeVariables {
                mode: FilterMode::Guarded,
                stability: None,
                stage: None,
            });
        let query = operation.query.to_string();
        for field in [
            "$mode: FilterMode!",
            "$stability: FrameCount",
            "$stage: StageName",
            "setFilterMode",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
        // The enum serializes in the schema's SCREAMING_SNAKE form.
        let variables = serde_json::to_value(&operation.variables).unwrap_or_default();
        assert_eq!(variables["mode"], "GUARDED");
    }
}
