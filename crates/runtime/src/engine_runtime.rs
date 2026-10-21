use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::watch;

use tangible_objects_engine::EngineEvent;
use tangible_objects_engine::EngineFrame;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSet;
use tangible_objects_model::PatternId;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::SetTracker;
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
            profiles: profiles.clone(),
            metrics: SessionMetrics::default(),
            message: None,
            #[cfg(feature = "gui")]
            gui_visible: true,
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

    /// Adds a registered profile to the shared profile list visible to
    /// interface readers.
    pub fn add_profile(&mut self, info: ProfileInfo) {
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.push(info);
        }
    }

    /// Removes a profile from the shared list by pattern ID.
    pub fn remove_profile(&mut self, id: PatternId) {
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.retain(|info| info.pattern_id != id);
        }
    }

    /// Publishes a processed frame: runs the set tracker, updates the
    /// snapshot and broadcasts engine events followed by set actions.
    pub fn publish(&mut self, frame: &EngineFrame, tracked: TrackedSet<'_>) {
        let actions = self.set_tracker.update(tracked);
        let touches_changed = frame
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::TouchPointsUpdated));
        self.metrics.record(
            (&frame.touches).into(),
            &frame.matches,
            tracked,
            touches_changed,
        );
        self.snapshot.send_replace(EngineSnapshot {
            tracked: tracked.to_vec(),
            touches: frame.touches.clone(),
            active_sets: self.set_tracker.active_set_ids(),
            device: self.device.clone(),
            recording: self.recording.clone(),
            completion: self.completion,
            field_sets: self.field_sets.clone(),
            metrics: self.metrics.clone(),
            message: self.message.clone(),
        });
        for event in &frame.events {
            self.broadcast(DomainEvent::Engine(event.clone()));
        }
        for action in actions {
            self.broadcast(DomainEvent::Set(action));
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
            metrics: self.metrics.clone(),
            message: self.message.clone(),
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

    /// Arms remote profile capture (`TrainProfile` command) - the
    /// device loops tick [`TrainingState`] with each frame's touches.
    pub fn start_training(&mut self, name: &str) {
        self.training = Some(TrainingState::new(name));
    }

    /// Advances remote training with this frame's touches; returns the
    /// captured [`ProfileFile`] once the contact set stayed stable
    /// (and clears the request).
    pub fn training_tick(
        &mut self,
        touches: &[TouchPoint],
    ) -> Option<tangible_objects_model::ProfileFile> {
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
    use tangible_objects_engine::EngineEvent;
    use tangible_objects_engine::EngineFrame;
    use tangible_objects_model::DEFAULT_SPAWN_STABILITY;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::SetAction;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TangibleSet;
    use tangible_objects_model::TangibleSetId;
    use tangible_objects_model::TrackedTangible;
    use tangible_objects_model::Vec2;

    fn tangible(id: u32, pattern: u32) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(id),
            pattern_id: PatternId::new(pattern),
            pattern_name: format!("fig-{pattern}").into(),
            center: Vec2::new(0.0, 0.0),
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
