use std::path::PathBuf;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
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

    /// Attached input device as a flat key-value list
    /// (`name`, `x_res`, `y_res`, `x_min`, `x_max`, `y_min`, `y_max`);
    /// empty values when no device/calibration is known.
    async fn device_info(&self) -> Vec<(String, String)> {
        let info = self.handle.snapshot().device;
        let Some(info) = info else {
            return Vec::new();
        };
        [
            ("name", info.name),
            ("x_res", info.x_res.map(|v| v.to_string())),
            ("y_res", info.y_res.map(|v| v.to_string())),
            ("x_min", info.x_min.map(|v| v.to_string())),
            ("x_max", info.x_max.map(|v| v.to_string())),
            ("y_min", info.y_min.map(|v| v.to_string())),
            ("y_max", info.y_max.map(|v| v.to_string())),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.map(|v| (key.to_string(), v)))
        .collect()
    }

    /// Live recording status as a flat key-value list; empty while no
    /// session is or was active.
    async fn recording_status(&self) -> Vec<(String, String)> {
        let status = self.handle.snapshot().recording;
        let Some(status) = status else {
            return Vec::new();
        };
        vec![
            ("active".to_string(), status.active.to_string()),
            (
                "label".to_string(),
                status
                    .label
                    .map(|label| label.to_string())
                    .unwrap_or_default(),
            ),
            (
                "recorded_frames".to_string(),
                status.recorded_frames.to_string(),
            ),
            (
                "dropped_frames".to_string(),
                status.dropped_frames.to_string(),
            ),
            (
                "dropped_events".to_string(),
                status.dropped_events.to_string(),
            ),
        ]
    }

    /// Switches the filter proxy's suppression policy
    /// (`off`/`immediate`/`gated`/`guarded`); `stability > 0` overrides
    /// the consecutive-matched-frames gate, `0` keeps it.
    async fn set_filter_mode(&self, mode: &str, stability: u64) -> fdo::Result<()> {
        let mode = mode.parse::<FilterMode>().map_err(|_| {
            fdo::Error::InvalidArgs(
                "mode must be one of: off, immediate, gated, guarded, tangibles-only".to_string(),
            )
        })?;
        let stability = (stability > 0).then(|| FrameCount::new(stability));
        self.handle
            .send_command(ControlCommand::SetFilterMode { mode, stability })
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
