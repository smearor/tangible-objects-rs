//! `UpdateProfileArgs` - arguments for the `update_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::StageName;

/// Arguments for the `update_profile` tool - replaces a registered
/// pattern's geometry while keeping its ID.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateProfileArgs {
    /// Pattern ID from `list_profiles`.
    pub pattern_id: PatternId,
    /// Figure name.
    pub name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    pub points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    pub tolerance: PatternTolerance,
    /// Whether a partial match may spawn this figure.
    pub spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation.
    pub spawn_stability: Option<u64>,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md); `null` keeps the pattern's stage.
    pub stage: Option<StageName>,
}
