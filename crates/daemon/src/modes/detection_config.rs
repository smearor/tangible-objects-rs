//! Optional detection settings bundled for the run_* modes.

use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSet;
use tangible_objects_model::Millimeters;

#[derive(Debug, Default, Clone)]
pub struct DetectionConfig {
    /// Edge ghost margin in millimeters (`--edge-filter-mm`).
    pub edge: Option<Millimeters>,
    /// Occupancy field sets (`--fields`).
    pub fields: Vec<FieldSet>,
    /// Missing-pin completion config (`--completion-*` flags).
    pub completion: CompletionConfig,
}
