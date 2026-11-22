//! The render model shared between the device thread (writer) and the
//! GTK main loop (reader) - the GUI's entire cross-thread contract.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use tangible_objects_engine::LegacyMode;
use tangible_objects_model::CompletionConfig;
use tangible_objects_runtime::ControlCommand;

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::Calibration;
use tangible_objects_model::CardFootprint;
use tangible_objects_model::ContactCount;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DeviceInfo;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldCount;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetCount;
use tangible_objects_model::FilterMode;
use tangible_objects_model::InputRegionDiagnostics;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::PatternName;
use tangible_objects_model::ProfileInfo;
use tangible_objects_model::SeatSession;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::SpellDiagnostics;
use tangible_objects_model::SpellMatched;
use tangible_objects_model::TangibleRenderMode;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::EngineHandle;

use crate::BackgroundAlpha;
use crate::GuiOptions;
use crate::calibration_source::CalibrationSource;
use crate::field_set_row::FieldSetRow;
use crate::gui_mode::GuiMode;
use crate::pattern_style::PatternStyle;

/// Snapshot the draw functions consume - written by the device thread
/// once per `SYN_REPORT` frame, read by the ~30 fps GTK timer.
pub struct SharedState {
    /// Active touch points.
    pub touches: Vec<TouchPoint>,
    /// Active contacts before ingest filtering (edge margin and dead
    /// zones) - `touches` holds the filtered set, so the dead-zone
    /// overlay's active indicator reads the suppressed positions here.
    pub unfiltered_touches: Vec<TouchPoint>,
    /// Raw pattern matches of the last frame (live view).
    pub matches: Vec<PatternMatch>,
    /// Currently tracked tangibles (pending flags render dimmed).
    pub tracked: Vec<TrackedTangible>,
    /// Effective axis calibration (mm conversion for the canvas scale).
    pub calibration: Option<Calibration>,
    /// Where the active calibration came from (device vs profile
    /// override) - `None` while uncalibrated.
    pub calibration_source: Option<CalibrationSource>,
    /// Accumulated session counters (Metrics card).
    pub metrics: SessionMetrics,
    /// Consecutive frames the contact set held the same size.
    pub stable_frames: u32,
    /// Contact set stable long enough for profile capture.
    pub stable: bool,
    /// Name of the registered profile the current contact set already
    /// matches - `Some` only in non-live modes, so the training card
    /// can warn about double captures.
    pub training_match: Option<PatternName>,
    /// Live vs. training visualization.
    pub mode: GuiMode,
    /// Device-open failure shown instead of the surface.
    pub device_error: Option<String>,
    /// Reported input-device identity + raw axis ranges (device card).
    pub device_info: Option<DeviceInfo>,
    /// User-facing message pushed via an interface (`ShowMessage`).
    pub message: Option<String>,
    /// Whether the window should be shown (`SetGuiVisible`).
    pub visible: bool,
    /// Display rotation the mm→px mapping applies (`--display-rotation`)
    /// - the evdev stream always speaks the panel's native frame.
    pub rotation: DisplayRotation,
    /// Canvas size in pixels - the draw callback writes it every
    /// frame; `filter_card_touches` needs it to map mm contacts onto
    /// card footprints.
    pub canvas_size: Vec2<Pixels>,
    /// Visible card hit areas in board-layer pixels - the passthrough
    /// tick collects them whether the input-region sync runs or not.
    /// Touches landing inside a card are GUI interactions and get
    /// filtered out of `touches` (a card tap must not disturb the
    /// training stability window).
    pub card_footprints: Vec<CardFootprint>,
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
    /// Spell-gesture overlay visible (`SetSpellsVisible`, SPELLS.md).
    pub spells_visible: bool,
    /// Live gesture trajectories per slot - debug polylines for the
    /// spell overlay.
    pub trajectories: Vec<Vec<Vec2<Millimeters>>>,
    /// Most recent spell match and when it fired - the match flash.
    pub last_spell: Option<(SpellMatched, Instant)>,
    /// Match evaluation of the last dispatched gesture episode -
    /// per-stroke Fréchet scores vs the spell's corridor plus the
    /// finger assignment (SPELLS.md's debug view). Retained for near
    /// misses so a rejected gesture shows how far it was off.
    pub spell_diagnostics: Option<SpellDiagnostics>,
    /// Repetitions an active spell training has captured - `None`
    /// while no `TrainSpell` session is armed (SPELLS.md's
    /// learn-by-repetition progress for the spell-training card).
    pub spell_training_progress: Option<usize>,
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
    /// Display rows of the registered field sets (name, owner,
    /// enabled, field count) - the field-sets card's data.
    pub field_set_rows: Vec<FieldSetRow>,
    /// Loaded and dynamically registered figure profiles - the
    /// profiles card's data (positions come from `tracked`).
    pub profiles: Vec<ProfileInfo>,
    /// Whether the ruler overlay is drawn
    /// (`SetRulerVisible`/`--ruler-visible`).
    pub ruler_visible: bool,
    /// Whether the grid overlay is drawn
    /// (`SetGridVisible`/`--grid-visible`).
    pub grid_visible: bool,
    /// Whether the positions overlay is drawn
    /// (`SetPositionsVisible`/`--positions-visible`).
    pub positions_visible: bool,
    /// Whether the seat overlay is drawn
    /// (`SetSeatsVisible`/`--seats-visible`,
    /// TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
    pub seats_visible: bool,
    /// Claimed seat sessions with their rosters - the seat overlay's
    /// and seats card's data.
    pub seat_sessions: Vec<SeatSession>,
    /// Whether touches pass through to the desktop except on GUI
    /// elements (`SetTouchPassthrough`/`--touch-passthrough`).
    pub touch_passthrough: bool,
    /// The passthrough input region as last applied to the surface -
    /// written by the region-sync timer, forwarded into the engine
    /// snapshot so interfaces (`input_region_diagnostics`) can compare
    /// footprints against rendered cards.
    pub input_region_diagnostics: InputRegionDiagnostics,
    /// Events observed by the window-level probe (capture-phase
    /// `EventControllerLegacy`), counted by event-type name. A tap on
    /// a card that increments nothing never reached the surface - the
    /// applied input region then differs from the diagnosed one.
    pub event_counts: BTreeMap<String, u64>,
    /// Surface-space position of the most recent probed touch or
    /// button event.
    pub last_event_position: Option<Vec2<Pixels>>,
    /// Selected suppression policy (`SetFilterMode` command +
    /// `--filter-mode` start value) - display state.
    pub filter_mode: FilterMode,
    /// Active missing-pin completion config (Parameters card).
    pub completion: CompletionConfig,
    /// Legacy single-touch event policy (Parameters card).
    pub legacy_mode: LegacyMode,
    /// Filter pipeline stability frames, `0` until a `SetFilterMode`
    /// command carries `stability` (Parameters card).
    pub filter_stability: u64,
    /// Draws the bounding box of every matched figure - debugging aid
    /// (GUI-local toggle, Parameters card).
    pub profile_bounds_visible: bool,
    /// Which layers the live view draws per matched tangible
    /// (GUI-local toggle on the visualization card).
    pub render_mode: TangibleRenderMode,
    /// Declared profile styles (color/shape) keyed like
    /// `engine.patterns()` - the draw loop colors figures and outlines
    /// physical shapes from it.
    pub pattern_styles: Vec<PatternStyle>,
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

/// Sends a `ControlCommand` over the runtime's command channel - the
/// same path the interface servers take. Uses `try_lock`: the
/// status-sync timer calls `set_active` on the toggle buttons while
/// holding `state`, which re-fires `connect_toggled` on this same
/// thread - a blocking `lock()` would deadlock on the non-recursive
/// mutex. Dropping the (idempotent) command on contention is safe.
pub(crate) fn send_command(state: &Arc<Mutex<SharedState>>, command: ControlCommand) {
    if let Ok(s) = state.try_lock()
        && let Some(handle) = &s.command_handle
    {
        handle.try_send_command(command).ok();
    }
}

/// Like [`send_command`], but builds the command's payload under
/// the same try-lock - for commands that merge live state (e.g.
/// `SetCompletion` carrying the untouched fields). A blocking
/// `state.lock()` inside a `connect_toggled` handler deadlocks
/// whenever a sync tick holds the guard while calling `set_active`
/// (non-recursive mutex, same thread).
pub(crate) fn send_command_built(
    state: &Arc<Mutex<SharedState>>,
    build: impl FnOnce(&SharedState) -> ControlCommand,
) {
    if let Ok(s) = state.try_lock()
        && let Some(handle) = &s.command_handle
    {
        handle.try_send_command(build(&s)).ok();
    }
}

impl SharedState {
    /// Initial state from the launch options - `SetGuiVisible` and
    /// friends toggle the flags afterwards.
    pub fn new(options: &GuiOptions) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            touches: Vec::new(),
            unfiltered_touches: Vec::new(),
            matches: Vec::new(),
            tracked: Vec::new(),
            calibration: None,
            calibration_source: None,
            metrics: SessionMetrics::default(),
            stable_frames: 0,
            stable: false,
            training_match: None,
            mode: GuiMode::Live,
            device_error: None,
            device_info: None,
            message: None,
            visible: options.visible,
            rotation: options.rotation,
            canvas_size: Vec2::ZERO,
            card_footprints: Vec::new(),
            background_alpha: options.background_alpha,
            background_alpha_request: None,
            dead_zones: Vec::new(),
            dead_zones_visible: options.dead_zones_visible,
            spells_visible: options.spells_visible,
            trajectories: Vec::new(),
            last_spell: None,
            spell_diagnostics: None,
            spell_training_progress: None,
            dead_zone_filtered: ContactCount::default(),
            field_set_count: FieldSetCount::default(),
            field_count: FieldCount::default(),
            occupied_fields: FieldCount::default(),
            edge_margin: Millimeters::new(0.0),
            edge_margin_visible: options.edge_margin_visible,
            fields_visible: options.fields_visible,
            field_sets: Vec::new(),
            field_set_rows: Vec::new(),
            profiles: Vec::new(),
            ruler_visible: options.ruler_visible,
            grid_visible: options.grid_visible,
            positions_visible: options.positions_visible,
            seats_visible: options.seats_visible,
            seat_sessions: Vec::new(),
            touch_passthrough: options.touch_passthrough,
            input_region_diagnostics: InputRegionDiagnostics::default(),
            event_counts: BTreeMap::new(),
            last_event_position: None,
            filter_mode: options.filter_mode,
            completion: CompletionConfig::default(),
            legacy_mode: LegacyMode::default(),
            filter_stability: 0,
            profile_bounds_visible: false,
            render_mode: TangibleRenderMode::default(),
            pattern_styles: Vec::new(),
            incoming_contacts: ContactCount::default(),
            edge_filtered: ContactCount::default(),
            mode_suppressed: ContactCount::default(),
            forwarded: ContactCount::default(),
            command_handle: None,
        }))
    }

    /// Drops contacts that land inside a visible card's footprint -
    /// card taps are GUI interactions, not surface figure points:
    /// they must not disturb the training stability window or draw a
    /// surface dot. No-op without calibration, canvas size or cards.
    pub fn filter_card_touches(&self, touches: &mut Vec<TouchPoint>) {
        let Some(cal) = self.calibration else {
            return;
        };
        if self.card_footprints.is_empty() || self.canvas_size.x <= 0.0 {
            return;
        }
        let canvas = self.canvas_size;
        let rotation = self.rotation;
        touches.retain(|touch| {
            let px = cal.mm_to_screen(touch.pos, canvas, rotation);
            !self
                .card_footprints
                .iter()
                .filter(|card| card.visible)
                .any(|card| card.contains(px))
        });
    }
}

/// Whether the shared state's mode runs the detection engine on
/// frames. `Live` and `SpellTraining` both do - spell training feeds
/// its episodes from `process_inner`; only `Training` (profile
/// capture) bypasses the engine for its stability window.
pub fn mode_runs_engine(state: &Arc<Mutex<SharedState>>) -> bool {
    state
        .lock()
        .map(|s| matches!(s.mode, GuiMode::Live | GuiMode::SpellTraining))
        .unwrap_or(true)
}
