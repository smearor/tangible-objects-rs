use crate::CompletionConfig;
use crate::FieldSet;
use crate::Millimeters;
use crate::Region;

/// Optional detection settings bundled for the daemon's run modes.
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
