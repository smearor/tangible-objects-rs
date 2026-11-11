use crate::CompletionConfig;
use crate::DeadZone;
use crate::FieldSet;
use crate::Millimeters;
use crate::Region;
use crate::Spell;

/// Optional detection settings bundled for the daemon's run modes.
#[derive(Debug, Clone)]
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
    /// Blacklisted surface positions - the startup dead-zone set
    /// (runtime-store reload plus static config).
    pub dead_zones: Vec<DeadZone>,
    /// Import the kernel's MT slot state at startup (`--no-import-slots`
    /// disables it) - contacts held open before the reader attached
    /// become visible to the engine immediately.
    pub import_slots: bool,
    /// Startup spell set (`spells.d/` runtime-store reload, SPELLS.md).
    pub spells: Vec<Spell>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            edge: None,
            fields: Vec::new(),
            completion: CompletionConfig::default(),
            zones: Vec::new(),
            dead_zones: Vec::new(),
            import_slots: true,
            spells: Vec::new(),
        }
    }
}
