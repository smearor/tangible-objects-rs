//! `AnalyzeSimilarityVariables` - the variables of
//! `analyzeSimilarity`.

use cynic::QueryVariables;

/// Variables of `analyzeSimilarity`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct AnalyzeSimilarityVariables {
    /// Profiles directory; `None` uses the daemon default.
    pub profiles_dir: Option<String>,
}
