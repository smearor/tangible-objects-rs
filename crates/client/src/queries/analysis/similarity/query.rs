//! `AnalyzeSimilarityQuery` - the cynic fragment for the
//! `analyzeSimilarity` query.

use cynic::QueryFragment;
use serde_json::Value;

use crate::queries::analysis::similarity::variables::AnalyzeSimilarityVariablesFields;

/// Root-field wrapper for `analyzeSimilarity` - the `JSON` scalar
/// selects as `serde_json::Value`; the response decodes into
/// `SimilarityReport`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "AnalyzeSimilarityVariables"
)]
pub(crate) struct AnalyzeSimilarityQuery {
    /// `analyzeSimilarity` field.
    #[cynic(rename = "analyzeSimilarity")]
    #[arguments(profilesDir: $profiles_dir)]
    analyze_similarity: Value,
}
