//! The render model shared between the device thread (writer) and the
//! GTK main loop (reader) - the GUI's entire cross-thread contract.

use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_model::Calibration;
use tangible_objects_model::ContactCount;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldCount;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetCount;
use tangible_objects_model::FilterMode;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::EngineHandle;

use crate::BackgroundAlpha;
use crate::GuiOptions;
use crate::gui_mode::GuiMode;

/// Snapshot the draw functions consume - written by the device thread
/// once per `SYN_REPORT` frame, read by the ~30 fps GTK timer.
pub struct SharedState {
    /// Active touch points.
    pub touches: Vec<TouchPoint>,
    /// Raw pattern matches of the last frame (live view).
    pub matches: Vec<PatternMatch>,
    /// Currently tracked tangibles (pending flags render dimmed).
    pub tracked: Vec<TrackedTangible>,
    /// Effective axis calibration (mm conversion for the canvas scale).
    pub calibration: Option<Calibration>,
    /// Consecutive frames the contact set held the same size.
    pub stable_frames: u32,
    /// Contact set stable long enough for profile capture.
    pub stable: bool,
    /// Live vs. training visualization.
    pub mode: GuiMode,
    /// Device-open failure shown instead of the surface.
    pub device_error: Option<String>,
    /// User-facing message pushed via an interface (`ShowMessage`).
    pub message: Option<String>,
    /// Whether the window should be shown (`SetGuiVisible`).
    pub visible: bool,
    /// Display rotation the mm→px mapping applies (`--display-rotation`)
    /// - the evdev stream always speaks the panel's native frame.
    pub rotation: DisplayRotation,
    /// Canvas background opacity (`--background-alpha`, the alpha
    /// toggle and `SetBackgroundAlpha` commands).
    pub background_alpha: BackgroundAlpha,
    /// Alpha change requested by the control-bar toggle - the device
    /// thread drains it into the runtime (`SetBackgroundAlpha`
    /// semantics) so the interface-visible state stays authoritative.
    pub background_alpha_request: Option<BackgroundAlpha>,
    /// Registered dead zones - the overlay data set.
    pub dead_zones: Vec<DeadZone>,
    /// Whether the dead-zone overlay is drawn
    /// (`SetDeadZonesVisible`/`--dead-zones-visible`).
    pub dead_zones_visible: bool,
    /// Contacts dropped by dead-zone filtering on the last frame.
    pub dead_zone_filtered: ContactCount,
    /// Registered field sets (snapshot `field_sets` count).
    pub field_set_count: FieldSetCount,
    /// Occupancy fields across all field sets.
    pub field_count: FieldCount,
    /// Fields currently occupied.
    pub occupied_fields: FieldCount,
    /// Edge margin in millimeters - the ingest-filtered strip at the
    /// surface edges.
    pub edge_margin: Millimeters,
    /// Whether the edge-margin overlay is drawn
    /// (`SetEdgeMarginVisible`/`--edge-margin-visible`).
    pub edge_margin_visible: bool,
    /// Whether the field-set overlay is drawn
    /// (`SetFieldsVisible`/`--fields-visible`).
    pub fields_visible: bool,
    /// Registered field sets - the overlay data set.
    pub field_sets: Vec<FieldSet>,
    /// Whether the ruler overlay is drawn
    /// (`SetRulerVisible`/`--ruler-visible`).
    pub ruler_visible: bool,
    /// Whether the positions overlay is drawn
    /// (`SetPositionsVisible`/`--positions-visible`).
    pub positions_visible: bool,
    /// Whether touches pass through to the desktop except on GUI
    /// elements (`SetTouchPassthrough`/`--touch-passthrough`).
    pub touch_passthrough: bool,
    /// Selected suppression policy (`SetFilterMode` command +
    /// `--filter-mode` start value) - display state.
    pub filter_mode: FilterMode,
    /// Active contacts before ingest filtering on the last frame.
    pub incoming_contacts: ContactCount,
    /// Touches dropped by edge-margin filtering on the last frame.
    pub edge_filtered: ContactCount,
    /// Touches the filter proxy would suppress under the current
    /// `filter_mode` (tangible-bound slots, `suppress` fields, or all
    /// contacts in `TangiblesOnly`) - virtual count, the GUI has no
    /// proxy.
    pub mode_suppressed: ContactCount,
    /// Touches the filter proxy would forward - `touches` minus
    /// `mode_suppressed`. Virtual count in `--gui` mode.
    pub forwarded: ContactCount,
    /// Command channel handle - the control-bar buttons issue the
    /// same `ControlCommand`s the interfaces do.
    pub command_handle: Option<EngineHandle>,
}

impl SharedState {
    /// Initial state from the launch options - `SetGuiVisible` and
    /// friends toggle the flags afterwards.
    pub fn new(options: &GuiOptions) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            touches: Vec::new(),
            matches: Vec::new(),
            tracked: Vec::new(),
            calibration: None,
            stable_frames: 0,
            stable: false,
            mode: GuiMode::Live,
            device_error: None,
            message: None,
            visible: options.visible,
            rotation: options.rotation,
            background_alpha: options.background_alpha,
            background_alpha_request: None,
            dead_zones: Vec::new(),
            dead_zones_visible: options.dead_zones_visible,
            dead_zone_filtered: ContactCount::default(),
            field_set_count: FieldSetCount::default(),
            field_count: FieldCount::default(),
            occupied_fields: FieldCount::default(),
            edge_margin: Millimeters::new(0.0),
            edge_margin_visible: options.edge_margin_visible,
            fields_visible: options.fields_visible,
            field_sets: Vec::new(),
            ruler_visible: options.ruler_visible,
            positions_visible: options.positions_visible,
            touch_passthrough: options.touch_passthrough,
            filter_mode: options.filter_mode,
            incoming_contacts: ContactCount::default(),
            edge_filtered: ContactCount::default(),
            mode_suppressed: ContactCount::default(),
            forwarded: ContactCount::default(),
            command_handle: None,
        }))
    }
}

/// Whether the shared state is in live mode.
pub fn mode_is_live(state: &Arc<Mutex<SharedState>>) -> bool {
    state
        .lock()
        .map(|s| s.mode == GuiMode::Live)
        .unwrap_or(true)
}
