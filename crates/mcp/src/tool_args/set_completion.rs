//! `SetCompletionArgs` - arguments for the `set_completion` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_completion` tool - every field optional, unset
/// fields keep the current value.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCompletionArgs {
    /// Completion mode: `match` (claim binds for tracking only) or
    /// `exact` (completed partial counts as exact for suppression).
    pub mode: Option<String>,
    /// Claim radius as a multiple of the pattern's scalar tolerance.
    pub radius_factor: Option<f64>,
    /// Maximum missing pins a partial may complete (1 or 2).
    pub max_missing: Option<u32>,
    /// Whether claimed slots enter the tangible's slot binding.
    pub persist: Option<bool>,
    /// Incumbent re-match tolerance widening (`match_score <= factor`,
    /// `1.0` disables).
    pub affinity_factor: Option<f64>,
    /// Per-slot position smoothing alpha (`0.0` disables).
    pub smoothing_alpha: Option<f64>,
}
