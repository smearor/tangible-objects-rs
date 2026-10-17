use serde::Deserialize;
use serde::Serialize;

use super::CompletionMode;

/// Configuration of missing-pin completion (TRACKING_STABILIZATION.md).
///
/// A partial match predicts the missing pins' expected positions from the
/// geometric transform of its matched pins and claims a nearby unassigned
/// touch within `radius_factor` × pattern tolerance.
///
/// Loadable from `~/.config/tangible-objects/completion.toml` and
/// changeable at runtime via the interfaces (GraphQL/MCP/D-Bus).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "async-graphql",
    derive(async_graphql::SimpleObject, async_graphql::InputObject),
    graphql(input_name = "CompletionInput")
)]
pub struct CompletionConfig {
    /// Claim radius as a multiple of the pattern's scalar tolerance.
    /// Default 2.0.
    pub radius_factor: f64,
    /// Maximum number of missing pins a partial may complete (1 or 2).
    /// Default 1.
    pub max_missing: u32,
    /// Whether claimed slots enter the tangible's slot binding for slot
    /// affinity in subsequent frames. Default `true`.
    pub persist: bool,
    /// Whether a completed partial counts as a match only or as exact for
    /// suppression. Default [`CompletionMode::Match`].
    pub mode: CompletionMode,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            radius_factor: 2.0,
            max_missing: 1,
            persist: true,
            mode: CompletionMode::Match,
        }
    }
}
