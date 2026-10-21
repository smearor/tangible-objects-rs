//! `CompletionInput` - the cynic input object for `setCompletion`,
//! a wire DTO of [`CompletionConfig`].

use tangible_objects_model::AffinityFactor;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::SmoothingAlpha;

/// The `CompletionInput` input object - wire DTO of
/// [`CompletionConfig`] (`max_missing` narrows to the `Int` scalar).
#[derive(cynic::InputObject)]
#[cynic(
    graphql_type = "CompletionInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct CompletionInput {
    /// Claim radius as a multiple of the pattern tolerance.
    radius_factor: f64,
    /// Maximum number of missing pins a partial may complete.
    max_missing: i32,
    /// Whether claimed slots join the slot binding.
    persist: bool,
    /// Whether a completed partial counts as match or exact.
    mode: CompletionMode,
    /// Incumbent re-match tolerance widening factor.
    affinity_factor: AffinityFactor,
    /// Per-slot position smoothing alpha.
    smoothing_alpha: SmoothingAlpha,
}

impl From<&CompletionConfig> for CompletionInput {
    /// Projects the domain config into the wire input shape.
    fn from(config: &CompletionConfig) -> Self {
        Self {
            radius_factor: config.radius_factor,
            max_missing: config.max_missing as i32,
            persist: config.persist,
            mode: config.mode,
            affinity_factor: config.affinity_factor,
            smoothing_alpha: config.smoothing_alpha,
        }
    }
}
