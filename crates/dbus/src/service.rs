use std::path::PathBuf;

use tangible_objects_model::RecordLabel;
use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::ProfileInfo;
use zbus::fdo;
use zbus::interface;
use zbus::object_server::SignalEmitter;

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
    async fn list_tangibles(&self) -> Vec<TrackedTangible> {
        self.handle.snapshot().tracked
    }

    /// Loaded figure profiles.
    async fn list_profiles(&self) -> Vec<ProfileInfo> {
        self.handle.profiles().to_vec()
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

    /// Starts a recording session writing `<base_path>.jsonl` +
    /// `<base_path>.evtest`; an empty `label` records unlabeled.
    async fn start_recording(&self, base_path: &str, label: &str) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(base_path),
                label: (!label.is_empty()).then(|| RecordLabel::new(label)),
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Stops the active recording session.
    async fn stop_recording(&self) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::StopRecording)
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Sets the ground-truth label for subsequent recorded frames; an empty
    /// string clears it.
    async fn set_record_label(&self, label: &str) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::SetRecordLabel(
                (!label.is_empty()).then(|| RecordLabel::new(label)),
            ))
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// A new tangible was detected and is now tracked.
    #[zbus(signal)]
    pub async fn tangible_added(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible's pose changed.
    #[zbus(signal)]
    pub async fn tangible_moved(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible expired.
    #[zbus(signal)]
    pub async fn tangible_removed(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// The set of active touch points changed.
    #[zbus(signal)]
    pub async fn touch_points_updated(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// All mandatory set members became present (TOS-ADD).
    #[zbus(signal)]
    pub async fn set_added(emitter: &SignalEmitter<'_>, set_id: TangibleSetId) -> zbus::Result<()>;

    /// The last mandatory set member disappeared (TOS-REM).
    #[zbus(signal)]
    pub async fn set_removed(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
    ) -> zbus::Result<()>;

    /// A member tangible appeared in an active set (TOS-OBJ-ADD).
    #[zbus(signal)]
    pub async fn object_added(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A member tangible disappeared (TOS-OBJ-REM).
    #[zbus(signal)]
    pub async fn object_removed(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    #[zbus(signal)]
    pub async fn object_moved(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;
}
