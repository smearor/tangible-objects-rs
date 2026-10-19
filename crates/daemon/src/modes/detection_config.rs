//! Optional detection settings bundled for the run_* modes.

use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;

#[derive(Debug, Default, Clone)]
pub struct DetectionConfig {
    /// Edge ghost margin in millimeters (`--edge-filter-mm`).
    pub edge: Option<Millimeters>,
    /// Occupancy field sets (`--fields`).
    pub fields: Vec<FieldSet>,
    /// Missing-pin completion config (`--completion-*` flags).
    pub completion: CompletionConfig,
    /// Masked areas in millimeters (`--zone`, repeatable) - contacts
    /// inside are suppressed in `--filter` mode.
    pub zones: Vec<Region>,
}
