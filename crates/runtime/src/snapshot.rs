#[cfg(feature = "gui")]
use tangible_objects_model::AlphaPercent;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::ContactCount;
use tangible_objects_model::DeadZone;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FilterMode;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::DeviceInfo;
use crate::RecordingStatus;

/// Point-in-time snapshot of the detection state.
///
/// Distributed via a `watch` channel: interfaces read the latest snapshot
/// for query-style requests (`list_tangibles`) without replaying the event
/// stream.
#[derive(Debug, Clone, Default)]
pub struct EngineSnapshot {
    /// Currently tracked tangibles (EMA-smoothed poses).
    pub tracked: Vec<TrackedTangible>,
    /// Active touch points of the last processed frame.
    pub touches: Vec<TouchPoint>,
    /// IDs of the currently active Tangible Object Sets.
    pub active_sets: Vec<TangibleSetId>,
    /// Attached input device - set once at session start.
    pub device: Option<DeviceInfo>,
    /// Recording session status, updated every frame while active.
    pub recording: Option<RecordingStatus>,
    /// Active missing-pin completion configuration - read-only view for
    /// interfaces (`set_completion` commands go through the device loop).
    pub completion: Option<CompletionConfig>,
    /// Active occupancy field sets - static (`--fields`) plus dynamically
    /// registered ones.
    pub field_sets: Vec<FieldSet>,
    /// Active dead zones - startup set plus dynamically registered ones.
    pub dead_zones: Vec<DeadZone>,
    /// Accumulated session recognition metrics - uptime, re-ID churn,
    /// match quality, per-slot contact stability.
    pub metrics: SessionMetrics,
    /// Contacts dropped by dead-zone filtering on the last processed
    /// frame - the ingest-level suppression count.
    pub dead_zone_filtered: ContactCount,
    /// User-facing message currently displayed in the GUI
    /// (`show_message` command).
    pub message: Option<String>,
    /// Whether the daemon-embedded GUI window is shown
    /// (`SetGuiVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub gui_visible: bool,
    /// Whether the GUI canvas draws the dead-zone overlay
    /// (`SetDeadZonesVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub dead_zones_visible: bool,
    /// Canvas background opacity (`SetBackgroundAlpha` command) -
    /// only present with the `gui` feature.
    #[cfg(feature = "gui")]
    pub background_alpha: AlphaPercent,
    /// Whether the GUI canvas draws the edge-margin overlay
    /// (`SetEdgeMarginVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub edge_margin_visible: bool,
    /// Whether the GUI canvas draws the field-set overlay
    /// (`SetFieldsVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub fields_visible: bool,
    /// Whether the GUI canvas draws the ruler overlay
    /// (`SetRulerVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub ruler_visible: bool,
    /// Whether the GUI canvas draws the grid overlay
    /// (`SetGridVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub grid_visible: bool,
    /// Whether the GUI canvas draws the positions overlay
    /// (`SetPositionsVisible` command) - only present with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub positions_visible: bool,
    /// Whether touches pass through to the desktop except on GUI
    /// elements (`SetTouchPassthrough` command) - only present with
    /// the `gui` feature.
    #[cfg(feature = "gui")]
    pub touch_passthrough: bool,
    /// Selected suppression policy (`SetFilterMode` command) -
    /// display intent on daemons without a proxy.
    pub filter_mode: FilterMode,
}
