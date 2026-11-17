//! `FindProfileArgs` - arguments for the `find_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::PatternTolerance;

/// Arguments for the `find_profile` tool - the candidate profile
/// geometry, a subset of `register_pattern`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindProfileArgs {
    /// Figure name.
    pub name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    pub points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    pub tolerance: PatternTolerance,
}
