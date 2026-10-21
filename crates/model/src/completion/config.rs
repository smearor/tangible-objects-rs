use serde::Deserialize;
use serde::Serialize;

use super::AffinityFactor;
use super::CompletionMode;
use super::SmoothingAlpha;
#[cfg(feature = "cynic")]
use crate::schema;

/// Configuration of missing-pin completion (TRACKING_STABILIZATION.md).
///
/// A partial match predicts the missing pins' expected positions from the
/// geometric transform of its matched pins and claims a nearby unassigned
/// touch within `radius_factor` × pattern tolerance.
///
/// Loadable from `~/.config/tangible-objects/completion.toml` and
/// changeable at runtime via the interfaces (GraphQL/MCP/D-Bus).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[cfg_attr(
    feature = "async-graphql",
    derive(async_graphql::SimpleObject, async_graphql::InputObject),
    graphql(input_name = "CompletionInput")
)]
// The read side (`CompletionConfig` object) is a manual QueryFragment
// below; the input side lives as `CompletionInput` in the client -
// input objects are wire DTOs, the domain type keeps its own serde.
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
    /// Incumbent re-match tolerance widening (`match_score <= factor`);
    /// `1.0` disables. Default 2.0.
    pub affinity_factor: AffinityFactor,
    /// Per-slot position smoothing alpha; `0.0` disables. Default 0.6.
    pub smoothing_alpha: SmoothingAlpha,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            radius_factor: 2.0,
            max_missing: 1,
            persist: true,
            mode: CompletionMode::Match,
            affinity_factor: AffinityFactor::default(),
            smoothing_alpha: SmoothingAlpha::default(),
        }
    }
}

#[cfg(feature = "cynic")]
impl cynic::QueryFragment for CompletionConfig {
    type SchemaType = schema::CompletionConfig;
    type VariablesFields = ();
    const TYPE: Option<&'static str> = Some("CompletionConfig");

    /// Read-side selection - `radiusFactor`, `maxMissing`, `persist`,
    /// `mode`, `affinityFactor`, `smoothingAlpha`. The input side is
    /// the derived `InputObject` (`CompletionInput`).
    fn query(
        builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>,
    ) {
        let mut builder = builder;
        let _ = builder.select_field::<schema::__fields::CompletionConfig::radiusFactor, f64>();
        let _ = builder.select_field::<schema::__fields::CompletionConfig::maxMissing, i32>();
        let _ = builder.select_field::<schema::__fields::CompletionConfig::persist, bool>();
        let _ = builder
            .select_field::<schema::__fields::CompletionConfig::mode, schema::CompletionMode>();
        let _ = builder.select_field::<schema::__fields::CompletionConfig::affinityFactor, schema::AffinityFactor>();
        let _ = builder.select_field::<schema::__fields::CompletionConfig::smoothingAlpha, schema::SmoothingAlpha>();
    }
}
