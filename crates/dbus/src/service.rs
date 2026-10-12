use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use zbus::fdo;
use zbus::interface;
use zbus::object_server::SignalEmitter;

use super::Profile;
use super::Tangible;

/// D-Bus service object served at `/org/tangibleobjects/Engine`.
///
/// Methods read the latest snapshot or forward control commands to the
/// device loop; signals are emitted via
/// [`emit_signal`](Self::emit_signal).
pub struct EngineService {
    handle: EngineHandle,
}

impl EngineService {
    /// Creates the service for the given runtime handle.
    pub fn new(handle: EngineHandle) -> Self {
        Self { handle }
    }
}

#[interface(name = "org.tangibleobjects.Engine1")]
impl EngineService {
    /// All currently tracked tangibles.
    async fn list_tangibles(&self) -> Vec<Tangible> {
        self.handle
            .snapshot()
            .tracked
            .iter()
            .map(Tangible::from)
            .collect()
    }

    /// Loaded figure profiles.
    async fn list_profiles(&self) -> Vec<Profile> {
        self.handle.profiles().iter().map(Profile::from).collect()
    }

    /// Enables or disables the tangible filter mode.
    async fn set_filter_mode(&self, enabled: bool) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::SetFilterMode(enabled))
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Starts a training session capturing a new profile with `name`.
    async fn train_profile(&self, name: &str) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::TrainProfile(name.to_string()))
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// A new tangible was detected and is now tracked.
    #[zbus(signal)]
    pub async fn tangible_added(
        emitter: &SignalEmitter<'_>,
        tangible: Tangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible's pose changed.
    #[zbus(signal)]
    pub async fn tangible_moved(
        emitter: &SignalEmitter<'_>,
        tangible: Tangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible expired.
    #[zbus(signal)]
    pub async fn tangible_removed(
        emitter: &SignalEmitter<'_>,
        tangible: Tangible,
    ) -> zbus::Result<()>;

    /// The set of active touch points changed.
    #[zbus(signal)]
    pub async fn touch_points_updated(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// All mandatory set members became present (TOS-ADD).
    #[zbus(signal)]
    pub async fn set_added(emitter: &SignalEmitter<'_>, set_id: &str) -> zbus::Result<()>;

    /// The last mandatory set member disappeared (TOS-REM).
    #[zbus(signal)]
    pub async fn set_removed(emitter: &SignalEmitter<'_>, set_id: &str) -> zbus::Result<()>;

    /// A member tangible appeared in an active set (TOS-OBJ-ADD).
    #[zbus(signal)]
    pub async fn object_added(
        emitter: &SignalEmitter<'_>,
        set_id: &str,
        tangible: Tangible,
    ) -> zbus::Result<()>;

    /// A member tangible disappeared (TOS-OBJ-REM).
    #[zbus(signal)]
    pub async fn object_removed(
        emitter: &SignalEmitter<'_>,
        set_id: &str,
        tangible: Tangible,
    ) -> zbus::Result<()>;

    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    #[zbus(signal)]
    pub async fn object_moved(
        emitter: &SignalEmitter<'_>,
        set_id: &str,
        tangible: Tangible,
    ) -> zbus::Result<()>;
}
