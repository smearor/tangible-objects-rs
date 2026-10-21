//! `RegisterPatternArgs` - arguments for the `register_pattern` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::StageName;

/// Arguments for the `register_pattern` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterPatternArgs {
    /// Figure name.
    pub name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    pub points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    pub tolerance: PatternTolerance,
    /// Optional owner/application name.
    pub owner: Option<Owner>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store). String + parse on purpose: `Lifetime` enum serde flips
    /// with the `cynic` feature.
    pub lifetime: Option<String>,
    /// Whether a partial match may spawn this figure
    /// (SPAWN_AS_PARTIAL.md); default `false` (exact-only spawns).
    pub spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation;
    /// default 15.
    pub spawn_stability: Option<u64>,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md); `null` = the general stage.
    pub stage: Option<StageName>,
}
