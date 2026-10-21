//! `SimilarityArgs` - arguments for the `analyze_similarity` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for `analyze_similarity` - a profile-directory analysis.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SimilarityArgs {
    /// Profile directory on the daemon host (default `profiles/`).
    pub profiles_dir: Option<String>,
}
