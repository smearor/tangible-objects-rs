use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::watch;

use tangible_objects_model::TangibleSet;

use super::ControlCommand;
use super::DomainEvent;
use super::EngineSnapshot;
use super::ProfileInfo;

/// Cloneable consumer-side handle for interface crates.
///
/// Provides event subscription, snapshot access and the control channel -
/// everything an interface needs without engine access.
#[derive(Clone)]
pub struct EngineHandle {
    /// Broadcast sender for subscribing to the domain event stream.
    events: broadcast::Sender<DomainEvent>,
    /// Watch receiver for the latest snapshot (`snapshot()`,
    /// `watch_snapshot()`).
    snapshot: watch::Receiver<EngineSnapshot>,
    /// Sending half of the control channel - interface commands reach
    /// the device loop through it.
    commands: mpsc::Sender<ControlCommand>,
    /// Static Tangible Object Set definitions loaded at session start.
    sets: Arc<Vec<TangibleSet>>,
    /// Profile list shared with the runtime - dynamic registrations are
    /// published through it.
    profiles: Arc<RwLock<Vec<ProfileInfo>>>,
}

impl EngineHandle {
    /// Creates the consumer-side channel ends - called by
    /// `EngineRuntime::with_capacity`, which pairs the halves.
    pub(super) fn new(
        events: broadcast::Sender<DomainEvent>,
        snapshot: watch::Receiver<EngineSnapshot>,
        commands: mpsc::Sender<ControlCommand>,
        sets: Arc<Vec<TangibleSet>>,
        profiles: Arc<RwLock<Vec<ProfileInfo>>>,
    ) -> Self {
        Self {
            events,
            snapshot,
            commands,
            sets,
            profiles,
        }
    }

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

    /// Info of the loaded plus dynamically registered figure profiles.
    pub fn profiles(&self) -> Vec<ProfileInfo> {
        self.profiles
            .read()
            .map(|profiles| profiles.clone())
            .unwrap_or_default()
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
