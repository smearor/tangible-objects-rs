use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::watch;

use smearor_commons_geometry::Millimeters;
use tangible_objects_engine::EngineEvent;
use tangible_objects_engine::EngineFrame;
use tangible_objects_engine::LegacyMode;
#[cfg(feature = "gui")]
use tangible_objects_model::AlphaPercent;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::ContactCount;
use tangible_objects_model::DeadZone;
#[cfg(feature = "gui")]
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::SetTracker;
use tangible_objects_model::Spell;
use tangible_objects_model::TangibleSet;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedSet;

use super::ControlCommand;
use super::DeviceInfo;
use super::DomainEvent;
use super::EngineHandle;
use super::EngineSnapshot;
use super::LifecycleEvent;
use super::ProfileInfo;
use super::RecordingStatus;
use super::TrainingState;

/// Default capacity of the broadcast event channel.
///
/// Lagging receivers drop the oldest events (`RecvError::Lagged`); a full
/// channel never blocks the device loop.
pub const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Capacity of the control command channel back to the device loop.
pub const COMMAND_CHANNEL_CAPACITY: usize = 32;

/// Daemon-side runtime: publishes frames, runs set tracking and receives
/// control commands.
///
/// Created once per detection session; the cloneable [`EngineHandle`] is
/// handed to interface servers.
pub struct EngineRuntime {
    /// Tangible Object Set tracker - computes set/object transitions
    /// from each published frame's tracked tangibles.
    set_tracker: SetTracker,
    /// Broadcast channel for `DomainEvent`s (engine + set events);
    /// lagging receivers lose the oldest events.
    events: broadcast::Sender<DomainEvent>,
    /// Watch channel carrying the latest snapshot for query-style
    /// interface requests.
    snapshot: watch::Sender<EngineSnapshot>,
    /// Receiving half of the control channel - drained by the device
    /// loop once per frame/timeout.
    commands: mpsc::Receiver<ControlCommand>,
    /// Device info embedded in every snapshot; set once at session
    /// start.
    device: Option<DeviceInfo>,
    /// Recording status embedded in every snapshot while a session is
    /// or was active.
    recording: Option<RecordingStatus>,
    /// Active completion configuration embedded in every snapshot.
    completion: Option<CompletionConfig>,
    /// Active field sets (static + dynamically registered) embedded in
    /// every snapshot.
    field_sets: Vec<FieldSet>,
    /// Active dead zones (runtime-store set + dynamically registered)
    /// embedded in every snapshot.
    dead_zones: Vec<DeadZone>,
    spells: Vec<Spell>,
    /// Contacts dropped by dead-zone filtering on the last processed
    /// frame - forwarded into every snapshot.
    dead_zone_filtered: ContactCount,
    /// Profile list shared with the handle - registrations at runtime
    /// must be visible to interface readers (`RwLock`, not `watch`).
    profiles: Arc<RwLock<Vec<ProfileInfo>>>,
    /// Accumulated session recognition metrics.
    metrics: SessionMetrics,
    /// User-facing message shown by the GUI (`ShowMessage` command).
    message: Option<String>,
    /// Whether the daemon-embedded GUI window is shown
    /// (`SetGuiVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    gui_visible: bool,
    /// Quarter turns the GUI canvas renders (`SetDisplayRotation`
    /// command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    display_rotation: DisplayRotation,
    /// Whether the GUI canvas draws the dead-zone overlay
    /// (`SetDeadZonesVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    dead_zones_visible: bool,
    /// Whether the GUI draws the live spell-gesture overlay
    /// (`SetSpellsVisible` command, SPELLS.md) - `gui` feature only.
    #[cfg(feature = "gui")]
    spells_visible: bool,
    /// Canvas background opacity (`SetBackgroundAlpha` command) -
    /// only with the `gui` feature.
    #[cfg(feature = "gui")]
    background_alpha: AlphaPercent,
    /// Whether the GUI canvas draws the edge-margin overlay
    /// (`SetEdgeMarginVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    edge_margin_visible: bool,
    /// Whether the GUI canvas draws the field-set overlay
    /// (`SetFieldsVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    fields_visible: bool,
    /// Whether the GUI canvas draws the ruler overlay
    /// (`SetRulerVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    ruler_visible: bool,
    /// Whether the GUI canvas draws the grid overlay
    /// (`SetGridVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    grid_visible: bool,
    /// Whether the GUI canvas draws the positions overlay
    /// (`SetPositionsVisible` command) - only with the `gui` feature.
    #[cfg(feature = "gui")]
    positions_visible: bool,
    /// Whether touches fall through to the desktop beneath except on
    /// GUI elements (`SetTouchPassthrough` command) - only with the
    /// `gui` feature.
    #[cfg(feature = "gui")]
    touch_passthrough: bool,
    /// Selected suppression policy (`SetFilterMode` command) - display
    /// state in `--gui`; honored by the proxy in `--filter`.
    filter_mode: FilterMode,
    /// Current edge-margin ghost filter width (Parameters card).
    edge_margin: Millimeters,
    /// Current legacy single-touch event policy (Parameters card).
    legacy_mode: LegacyMode,
    /// Frame-count stability the filter pipeline was last configured
    /// with (Parameters card; `None` until a `SetFilterMode` carries
    /// `stability`).
    filter_stability: Option<FrameCount>,
    /// Pending remote profile capture (`TrainProfile` command) -
    /// the device loops tick it with each frame's touches.
    training: Option<TrainingState>,
}

impl EngineRuntime {
    /// Creates the runtime with a set tracker for the given definitions and
    /// the profile info of the loaded figure patterns.
    ///
    /// Returns the runtime (daemon side) and the handle handed to interface
    /// crates.
    pub fn new(sets: Vec<TangibleSet>, profiles: Vec<ProfileInfo>) -> (Self, EngineHandle) {
        Self::with_capacity(
            sets,
            profiles,
            EVENT_CHANNEL_CAPACITY,
            COMMAND_CHANNEL_CAPACITY,
        )
    }

    /// Like [`Self::new`] with explicit channel capacities.
    pub fn with_capacity(
        sets: Vec<TangibleSet>,
        profiles: Vec<ProfileInfo>,
        event_capacity: usize,
        command_capacity: usize,
    ) -> (Self, EngineHandle) {
        let (events, _) = broadcast::channel(event_capacity);
        let (snapshot_tx, snapshot_rx) = watch::channel(EngineSnapshot::default());
        let (command_tx, commands) = mpsc::channel(command_capacity);
        let profiles = Arc::new(RwLock::new(profiles));
        let runtime = Self {
            set_tracker: SetTracker::new(sets.clone()),
            events: events.clone(),
            snapshot: snapshot_tx,
            commands,
            device: None,
            recording: None,
            completion: None,
            field_sets: Vec::new(),
            dead_zones: Vec::new(),
            spells: Vec::new(),
            dead_zone_filtered: ContactCount::default(),
            profiles: profiles.clone(),
            metrics: SessionMetrics::default(),
            message: None,
            #[cfg(feature = "gui")]
            gui_visible: true,
            #[cfg(feature = "gui")]
            dead_zones_visible: false,
            #[cfg(feature = "gui")]
            spells_visible: false,
            #[cfg(feature = "gui")]
            background_alpha: AlphaPercent::default(),
            #[cfg(feature = "gui")]
            edge_margin_visible: false,
            #[cfg(feature = "gui")]
            fields_visible: false,
            #[cfg(feature = "gui")]
            ruler_visible: false,
            #[cfg(feature = "gui")]
            grid_visible: false,
            #[cfg(feature = "gui")]
            positions_visible: false,
            #[cfg(feature = "gui")]
            touch_passthrough: false,
            filter_mode: FilterMode::default(),
            edge_margin: Millimeters::new(0.0),
            legacy_mode: LegacyMode::default(),
            filter_stability: None,
            #[cfg(feature = "gui")]
            display_rotation: DisplayRotation::default(),
            training: None,
        };
        let handle = EngineHandle::new(events, snapshot_rx, command_tx, Arc::new(sets), profiles);
        (runtime, handle)
    }

    /// Sets the device info published in every subsequent snapshot.
    ///
    /// Call once at session start - the device does not change mid-session.
    pub fn set_device_info(&mut self, info: DeviceInfo) {
        self.device = Some(info);
    }

    /// Updates the recording status published in every subsequent snapshot;
    /// `None` clears it (no session).
    pub fn set_recording_status(&mut self, status: Option<RecordingStatus>) {
        self.recording = status;
    }

    /// Publishes the active completion configuration in subsequent
    /// snapshots. Set at session start and after each applied
    /// `SetCompletion` command.
    pub fn set_completion(&mut self, config: CompletionConfig) {
        self.completion = Some(config);
    }

    /// Publishes the active field sets (static plus dynamically
    /// registered) in subsequent snapshots.
    pub fn set_field_sets(&mut self, field_sets: Vec<FieldSet>) {
        self.field_sets = field_sets;
    }

    /// Publishes the active dead zones (startup set plus dynamically
    /// registered) in subsequent snapshots.
    pub fn set_dead_zones(&mut self, dead_zones: Vec<DeadZone>) {
        self.dead_zones = dead_zones;
    }

    /// Publishes the spell list (mirrors `engine.spells()`).
    pub fn set_spells(&mut self, spells: Vec<Spell>) {
        self.spells = spells;
    }

    /// Adds a registered profile to the shared profile list visible to
    /// interface readers.
    pub fn add_profile(&mut self, info: ProfileInfo) {
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.push(info);
        }
    }

    /// Removes a profile from the shared list by pattern ID.
    /// Updates the published enabled flag of a profile - the drain
    /// calls it after `SetProfileEnabled` applied to the engine.
    pub fn set_profile_enabled(&mut self, id: PatternId, enabled: bool) {
        if let Ok(mut profiles) = self.profiles.write()
            && let Some(info) = profiles.iter_mut().find(|info| info.pattern_id == id)
        {
            info.enabled = enabled;
        }
    }

    /// Removes the published profile entry for `id`.
    pub fn remove_profile(&mut self, id: PatternId) {
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.retain(|info| info.pattern_id != id);
        }
    }

    /// Publishes a processed frame: runs the set tracker, updates the
    /// snapshot and broadcasts engine events followed by set actions.
    pub fn publish(&mut self, frame: &EngineFrame, tracked: TrackedSet<'_>) {
        let actions = self.set_tracker.update(tracked);
        self.dead_zone_filtered = frame.dead_zone_filtered;
        let touches_changed = frame
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::TouchPointsUpdated));
        self.metrics.record(
            (&frame.touches).into(),
            &frame.matches,
            tracked,
            touches_changed,
            frame.ran_generic,
            frame.candidates_tested,
        );
        self.snapshot.send_replace(EngineSnapshot {
            tracked: tracked.to_vec(),
            touches: frame.touches.clone(),
            active_sets: self.set_tracker.active_set_ids(),
            device: self.device.clone(),
            recording: self.recording.clone(),
            completion: self.completion,
            field_sets: self.field_sets.clone(),
            dead_zones: self.dead_zones.clone(),
            spells: self.spells.clone(),
            dead_zone_filtered: self.dead_zone_filtered,
            metrics: self.metrics.clone(),
            message: self.message.clone(),
            #[cfg(feature = "gui")]
            gui_visible: self.gui_visible,
            #[cfg(feature = "gui")]
            dead_zones_visible: self.dead_zones_visible,
            #[cfg(feature = "gui")]
            spells_visible: self.spells_visible,
            #[cfg(feature = "gui")]
            background_alpha: self.background_alpha,
            #[cfg(feature = "gui")]
            edge_margin_visible: self.edge_margin_visible,
            #[cfg(feature = "gui")]
            fields_visible: self.fields_visible,
            #[cfg(feature = "gui")]
            ruler_visible: self.ruler_visible,
            #[cfg(feature = "gui")]
            grid_visible: self.grid_visible,
            #[cfg(feature = "gui")]
            positions_visible: self.positions_visible,
            #[cfg(feature = "gui")]
            touch_passthrough: self.touch_passthrough,
            filter_mode: self.filter_mode,
        });
        for event in &frame.events {
            self.broadcast(DomainEvent::Engine(event.clone()));
        }
        for action in actions {
            self.broadcast(DomainEvent::Set(action));
        }
        // SPELLS.md: completed gesture episodes surface as lifecycle
        // events - every subscription (GraphQL/D-Bus/MCP/TUI) sees
        // them without a per-mode drain.
        for matched in &frame.spell_matches {
            self.emit_lifecycle(LifecycleEvent::SpellMatched {
                name: matched.name.clone(),
                center: matched.center,
                score: matched.score,
            });
        }
    }

    /// Publishes the current engine state without a frame - used on the
    /// command-drain wake-up path so interface-visible state (e.g. a stopped
    /// recording) reaches clients even while the device is idle.
    pub fn publish_snapshot(&mut self, touches: Vec<TouchPoint>, tracked: TrackedSet<'_>) {
        let actions = self.set_tracker.update(tracked);
        self.snapshot.send_replace(EngineSnapshot {
            tracked: tracked.to_vec(),
            touches,
            active_sets: self.set_tracker.active_set_ids(),
            device: self.device.clone(),
            recording: self.recording.clone(),
            completion: self.completion,
            field_sets: self.field_sets.clone(),
            dead_zones: self.dead_zones.clone(),
            spells: self.spells.clone(),
            dead_zone_filtered: self.dead_zone_filtered,
            metrics: self.metrics.clone(),
            message: self.message.clone(),
            #[cfg(feature = "gui")]
            gui_visible: self.gui_visible,
            #[cfg(feature = "gui")]
            dead_zones_visible: self.dead_zones_visible,
            #[cfg(feature = "gui")]
            spells_visible: self.spells_visible,
            #[cfg(feature = "gui")]
            background_alpha: self.background_alpha,
            #[cfg(feature = "gui")]
            edge_margin_visible: self.edge_margin_visible,
            #[cfg(feature = "gui")]
            fields_visible: self.fields_visible,
            #[cfg(feature = "gui")]
            ruler_visible: self.ruler_visible,
            #[cfg(feature = "gui")]
            grid_visible: self.grid_visible,
            #[cfg(feature = "gui")]
            positions_visible: self.positions_visible,
            #[cfg(feature = "gui")]
            touch_passthrough: self.touch_passthrough,
            filter_mode: self.filter_mode,
        });
        for action in actions {
            self.broadcast(DomainEvent::Set(action));
        }
    }

    /// Accumulated session metrics (live view).
    pub fn metrics(&self) -> &SessionMetrics {
        &self.metrics
    }

    /// Resets the session metric counters.
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
    }

    /// Sets the user-facing message shown by the GUI; `None` clears it.
    pub fn set_message(&mut self, message: Option<String>) {
        self.message = message;
    }

    /// The current user-facing message, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Shows or hides the daemon-embedded GUI window
    /// (`SetGuiVisible` command); embedded in snapshots so the GUI
    /// polls it like `message`. Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_gui_visible(&mut self, visible: bool) {
        self.gui_visible = visible;
    }

    /// Whether the daemon-embedded GUI window should be shown.
    /// Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn gui_visible(&self) -> bool {
        self.gui_visible
    }

    /// Quarter turns the GUI canvas applies (`SetDisplayRotation`
    /// command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_display_rotation(&mut self, rotation: DisplayRotation) {
        self.display_rotation = rotation;
    }

    /// The GUI canvas rotation. Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn display_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Shows or hides the dead-zone overlay on the GUI canvas
    /// (`SetDeadZonesVisible` command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_dead_zones_visible(&mut self, visible: bool) {
        self.dead_zones_visible = visible;
    }

    /// Whether the GUI canvas draws the dead-zone overlay. Only with
    /// the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn dead_zones_visible(&self) -> bool {
        self.dead_zones_visible
    }

    /// Shows or hides the live spell-gesture overlay on the GUI
    /// canvas (`SetSpellsVisible` command, SPELLS.md). Only with the
    /// `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_spells_visible(&mut self, visible: bool) {
        self.spells_visible = visible;
    }

    /// Whether the GUI canvas draws the live spell-gesture overlay.
    /// Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn spells_visible(&self) -> bool {
        self.spells_visible
    }

    /// Sets the canvas background opacity (`SetBackgroundAlpha`
    /// command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_background_alpha(&mut self, percent: AlphaPercent) {
        self.background_alpha = percent;
    }

    /// Canvas background opacity. Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn background_alpha(&self) -> AlphaPercent {
        self.background_alpha
    }

    /// Sets the edge-margin overlay visibility
    /// (`SetEdgeMarginVisible` command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_edge_margin_visible(&mut self, visible: bool) {
        self.edge_margin_visible = visible;
    }

    /// Whether the GUI canvas draws the edge-margin overlay. Only
    /// with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn edge_margin_visible(&self) -> bool {
        self.edge_margin_visible
    }

    /// Sets the field-set overlay visibility (`SetFieldsVisible`
    /// command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_fields_visible(&mut self, visible: bool) {
        self.fields_visible = visible;
    }

    /// Whether the GUI canvas draws the field-set overlay. Only with
    /// the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn fields_visible(&self) -> bool {
        self.fields_visible
    }

    /// Sets the ruler overlay visibility (`SetRulerVisible` command).
    /// Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_ruler_visible(&mut self, visible: bool) {
        self.ruler_visible = visible;
    }

    /// Whether the GUI canvas draws the ruler overlay. Only with the
    /// `gui` feature.
    #[cfg(feature = "gui")]
    pub fn ruler_visible(&self) -> bool {
        self.ruler_visible
    }

    /// Sets the grid overlay visibility (`SetGridVisible` command).
    /// Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.grid_visible = visible;
    }

    /// Whether the GUI canvas draws the grid overlay. Only with the
    /// `gui` feature.
    #[cfg(feature = "gui")]
    pub fn grid_visible(&self) -> bool {
        self.grid_visible
    }

    /// Sets the positions overlay visibility (`SetPositionsVisible`
    /// command). Only with the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn set_positions_visible(&mut self, visible: bool) {
        self.positions_visible = visible;
    }

    /// Whether the GUI canvas draws the positions overlay. Only with
    /// the `gui` feature.
    #[cfg(feature = "gui")]
    pub fn positions_visible(&self) -> bool {
        self.positions_visible
    }

    /// Sets whether touches pass through to the desktop except on GUI
    /// elements (`SetTouchPassthrough` command). Only with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub fn set_touch_passthrough(&mut self, enabled: bool) {
        self.touch_passthrough = enabled;
    }

    /// Whether touch passthrough is active. Only with the `gui`
    /// feature.
    #[cfg(feature = "gui")]
    pub fn touch_passthrough(&self) -> bool {
        self.touch_passthrough
    }

    /// Sets the displayed filter mode (`SetFilterMode` command -
    /// display state in `--gui`; honored by the proxy in `--filter`).
    pub fn set_filter_mode(&mut self, mode: FilterMode) {
        self.filter_mode = mode;
    }

    /// The selected suppression policy.
    pub fn filter_mode(&self) -> FilterMode {
        self.filter_mode
    }

    /// Publishes the edge-margin ghost filter width (Parameters card).
    pub fn set_edge_margin(&mut self, margin: Millimeters) {
        self.edge_margin = margin;
    }

    /// The currently published edge-margin filter width.
    pub fn edge_margin(&self) -> Millimeters {
        self.edge_margin
    }

    /// Publishes the legacy single-touch event policy (Parameters
    /// card); the `--filter` pipeline applies it on its boundary.
    pub fn set_legacy_mode(&mut self, mode: LegacyMode) {
        self.legacy_mode = mode;
    }

    /// The currently published legacy event policy.
    pub fn legacy_mode(&self) -> LegacyMode {
        self.legacy_mode
    }

    /// Publishes the last stability frame-count handed to the filter
    /// pipeline via `SetFilterMode` (Parameters card).
    pub fn set_filter_stability(&mut self, stability: FrameCount) {
        self.filter_stability = Some(stability);
    }

    /// The last published filter stability, `None` when unset.
    pub fn filter_stability(&self) -> Option<FrameCount> {
        self.filter_stability
    }

    /// Arms remote profile capture (`TrainProfile` command) - the
    /// device loops tick [`TrainingState`] with each frame's touches.
    pub fn start_training(&mut self, name: &PatternName) {
        self.training = Some(TrainingState::new(name));
    }

    /// Advances remote training with this frame's touches; returns the
    /// captured [`ProfileFile`] once the contact set stayed stable
    /// (and clears the request).
    pub fn training_tick(&mut self, touches: &[TouchPoint]) -> Option<ProfileFile> {
        let state = self.training.as_mut()?;
        let profile = state.tick(touches)?;
        self.training = None;
        Some(profile)
    }

    /// Next pending control command from an interface, if any.
    ///
    /// Non-blocking; intended to be drained once per frame in the
    /// synchronous device loop.
    pub fn try_recv_command(&mut self) -> Result<ControlCommand, TryRecvError> {
        self.commands.try_recv()
    }

    /// Emits a [`LifecycleEvent`] to all domain-event subscribers.
    ///
    /// Called by the device loop when a control command *applies* - the
    /// event means "this actually took effect", so emit it only after the
    /// engine accepted the change (SUBSCRIPTIONS.md).
    pub fn emit_lifecycle(&self, event: LifecycleEvent) {
        self.broadcast(DomainEvent::Lifecycle(event));
    }

    /// Broadcasts a domain event to all subscribers.
    ///
    /// Zero receivers is the expected steady state when no interface is
    /// attached; a full channel drops the oldest event for lagging
    /// receivers but never blocks the device loop.
    fn broadcast(&self, event: DomainEvent) {
        let _ = self.events.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::EngineRuntime;
    use crate::ControlCommand;
    use crate::DomainEvent;
    use smearor_commons_geometry::Millimeters;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;
    use tangible_objects_engine::EngineEvent;
    use tangible_objects_engine::EngineFrame;
    use tangible_objects_model::ContactCount;
    use tangible_objects_model::DEFAULT_SPAWN_STABILITY;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::LifecycleEvent;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::SetAction;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::SpellMatched;
    use tangible_objects_model::SpellName;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TangibleSet;
    use tangible_objects_model::TangibleSetId;
    use tangible_objects_model::TrackedTangible;

    fn tangible(id: u32, pattern: u32) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(id),
            pattern_id: PatternId::new(pattern),
            pattern_name: format!("fig-{pattern}").into(),
            center: Vec2::ZERO,
            rotation_rad: RotationRad::new(0.0),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            frames_since_match: FrameCount::new(0),
        }
    }

    fn set(id: u32, mandatory: &[u32]) -> TangibleSet {
        TangibleSet {
            id: TangibleSetId::new(id),
            name: format!("set-{id}"),
            mandatory: mandatory.iter().map(|p| PatternId::new(*p)).collect(),
            additional: Vec::new(),
        }
    }

    fn frame(events: Vec<EngineEvent>) -> EngineFrame {
        EngineFrame {
            matches: Vec::new(),
            touches: Vec::new(),
            events,
            candidates: Vec::new(),
            spell_matches: Vec::new(),
            gesture_slots: SlotSet::default(),
            consumed_slots: SlotSet::default(),
            ran_generic: true,
            candidates_tested: 0.into(),
            dead_zone_filtered: ContactCount::default(),
        }
    }

    #[test]
    fn publish_broadcasts_engine_events_then_set_actions() {
        let tracked = vec![tangible(1, 1)];
        let (mut runtime, handle) = EngineRuntime::new(vec![set(1, &[1])], Vec::new());
        let mut receiver = handle.subscribe();

        runtime.publish(
            &frame(vec![
                EngineEvent::TangibleAdded(tracked[0].clone()),
                EngineEvent::TouchPointsUpdated,
            ]),
            (&tracked).into(),
        );

        assert!(matches!(
            receiver.try_recv(),
            Ok(DomainEvent::Engine(EngineEvent::TangibleAdded(_)))
        ));
        assert!(matches!(
            receiver.try_recv(),
            Ok(DomainEvent::Engine(EngineEvent::TouchPointsUpdated))
        ));
        assert!(matches!(
            receiver.try_recv(),
            Ok(DomainEvent::Set(SetAction::SetAdded(_)))
        ));
        assert!(matches!(
            receiver.try_recv(),
            Ok(DomainEvent::Set(SetAction::ObjectAdded(_, _)))
        ));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn publish_emits_spell_matched_lifecycle() {
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());
        let mut receiver = handle.subscribe();

        let mut frame = frame(Vec::new());
        frame.spell_matches.push(SpellMatched {
            name: SpellName::new("V"),
            center: Vec2::new(100.0, 80.0),
            score: Millimeters::new(3.5),
        });
        runtime.publish(&frame, (&Vec::new()).into());

        assert!(matches!(
            receiver.try_recv(),
            Ok(DomainEvent::Lifecycle(LifecycleEvent::SpellMatched { ref name, .. }))
                if name.as_str() == "V"
        ));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn publish_updates_snapshot() {
        let tracked = vec![tangible(1, 1), tangible(2, 2)];
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());

        runtime.publish(&frame(Vec::new()), (&tracked).into());

        let snapshot = handle.snapshot();
        assert_eq!(snapshot.tracked.len(), 2);
        assert_eq!(snapshot.tracked[0].id, TangibleId::new(1));
    }

    #[test]
    fn control_commands_reach_the_runtime() {
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());

        handle
            .try_send_command(ControlCommand::SetFilterMode {
                mode: tangible_objects_model::FilterMode::Guarded,
                stability: Some(tangible_objects_model::FrameCount::new(3)),
                stage: None,
            })
            .unwrap();

        assert!(matches!(
            runtime.try_recv_command(),
            Ok(ControlCommand::SetFilterMode {
                mode: tangible_objects_model::FilterMode::Guarded,
                stability: Some(_),
                stage: None,
            })
        ));
    }

    #[test]
    fn inactive_set_produces_no_actions() {
        // Pattern 2 is no member of the set (mandatory is pattern 1).
        let tracked = vec![tangible(1, 2)];
        let (mut runtime, handle) = EngineRuntime::new(vec![set(1, &[1])], Vec::new());
        let mut receiver = handle.subscribe();

        runtime.publish(&frame(Vec::new()), (&tracked).into());

        assert!(receiver.try_recv().is_err());
    }
}
