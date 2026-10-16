use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::watch;

use tangible_objects_engine::EngineFrame;
use tangible_objects_model::SetTracker;
use tangible_objects_model::TangibleSet;
use tangible_objects_model::TrackedTangible;

use super::ControlCommand;
use super::DomainEvent;
use super::EngineSnapshot;
use super::ProfileInfo;

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
    set_tracker: SetTracker,
    events: broadcast::Sender<DomainEvent>,
    snapshot: watch::Sender<EngineSnapshot>,
    commands: mpsc::Receiver<ControlCommand>,
}

/// Cloneable consumer-side handle for interface crates.
///
/// Provides event subscription, snapshot access and the control channel —
/// everything an interface needs without engine access.
#[derive(Clone)]
pub struct EngineHandle {
    events: broadcast::Sender<DomainEvent>,
    snapshot: watch::Receiver<EngineSnapshot>,
    commands: mpsc::Sender<ControlCommand>,
    sets: std::sync::Arc<Vec<TangibleSet>>,
    profiles: std::sync::Arc<Vec<ProfileInfo>>,
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
        let runtime = Self {
            set_tracker: SetTracker::new(sets.clone()),
            events: events.clone(),
            snapshot: snapshot_tx,
            commands,
        };
        let handle = EngineHandle {
            events,
            snapshot: snapshot_rx,
            commands: command_tx,
            sets: std::sync::Arc::new(sets),
            profiles: std::sync::Arc::new(profiles),
        };
        (runtime, handle)
    }

    /// Publishes a processed frame: runs the set tracker, updates the
    /// snapshot and broadcasts engine events followed by set actions.
    pub fn publish(&mut self, frame: &EngineFrame, tracked: &[TrackedTangible]) {
        let actions = self.set_tracker.update(tracked);
        self.snapshot.send_replace(EngineSnapshot {
            tracked: tracked.to_vec(),
            touches: frame.touches.clone(),
            active_sets: self.set_tracker.active_set_ids(),
        });
        for event in &frame.events {
            self.broadcast(DomainEvent::Engine(event.clone()));
        }
        for action in actions {
            self.broadcast(DomainEvent::Set(action));
        }
    }

    /// Next pending control command from an interface, if any.
    ///
    /// Non-blocking; intended to be drained once per frame in the
    /// synchronous device loop.
    pub fn try_recv_command(&mut self) -> Result<ControlCommand, TryRecvError> {
        self.commands.try_recv()
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

impl EngineHandle {
    /// Subscribes to the domain event stream.
    ///
    /// Each call returns an independent receiver; lagging receivers drop
    /// the oldest events.
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.events.subscribe()
    }

    /// The latest published snapshot.
    pub fn snapshot(&self) -> EngineSnapshot {
        self.snapshot.borrow().clone()
    }

    /// Watches the snapshot for change notifications.
    pub fn watch_snapshot(&self) -> watch::Receiver<EngineSnapshot> {
        self.snapshot.clone()
    }

    /// The configured Tangible Object Set definitions.
    pub fn sets(&self) -> &[TangibleSet] {
        &self.sets
    }

    /// Static info of the loaded figure profiles.
    pub fn profiles(&self) -> &[ProfileInfo] {
        &self.profiles
    }

    /// Sends a control command to the daemon's device loop.
    pub async fn send_command(
        &self,
        command: ControlCommand,
    ) -> Result<(), SendError<ControlCommand>> {
        self.commands.send(command).await
    }

    /// Non-blocking variant of [`Self::send_command`].
    pub fn try_send_command(
        &self,
        command: ControlCommand,
    ) -> Result<(), TrySendError<ControlCommand>> {
        self.commands.try_send(command)
    }
}

#[cfg(test)]
mod tests {
    use super::EngineRuntime;
    use crate::ControlCommand;
    use crate::DomainEvent;
    use tangible_objects_engine::EngineEvent;
    use tangible_objects_engine::EngineFrame;
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
            pattern_name: format!("fig-{pattern}"),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            frames_since_match: 0,
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
            &tracked,
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

        runtime.publish(&frame(Vec::new()), &tracked);

        let snapshot = handle.snapshot();
        assert_eq!(snapshot.tracked.len(), 2);
        assert_eq!(snapshot.tracked[0].id, TangibleId::new(1));
    }

    #[test]
    fn control_commands_reach_the_runtime() {
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());

        handle
            .try_send_command(ControlCommand::SetFilterMode(true))
            .unwrap();

        assert!(matches!(
            runtime.try_recv_command(),
            Ok(ControlCommand::SetFilterMode(true))
        ));
    }

    #[test]
    fn inactive_set_produces_no_actions() {
        // Pattern 2 is no member of the set (mandatory is pattern 1).
        let tracked = vec![tangible(1, 2)];
        let (mut runtime, handle) = EngineRuntime::new(vec![set(1, &[1])], Vec::new());
        let mut receiver = handle.subscribe();

        runtime.publish(&frame(Vec::new()), &tracked);

        assert!(receiver.try_recv().is_err());
    }
}
