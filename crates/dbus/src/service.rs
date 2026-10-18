use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::ProfileInfo;
use zbus::fdo;
use zbus::interface;
use zbus::message::Header;
use zbus::names::UniqueName;
use zbus::object_server::SignalEmitter;

/// Maps a caller's unique bus name to the owner strings it registered
/// under - drives automatic cleanup when the name vanishes.
pub type OwnerSessions = Arc<Mutex<HashMap<String, HashSet<String>>>>;

/// D-Bus service object served at `/org/tangibleobjects/Engine`.
///
/// Methods read the latest snapshot or forward control commands to the
/// device loop; signals are emitted via
/// [`emit_signal`](Self::emit_signal).
pub struct EngineService {
    handle: EngineHandle,
    /// Owner sessions for disconnect cleanup (`NameOwnerChanged`).
    owner_sessions: OwnerSessions,
}

impl EngineService {
    /// Creates the service for the given runtime handle and the shared
    /// owner-session map watched by [`serve`](crate::serve).
    pub fn new(handle: EngineHandle, owner_sessions: OwnerSessions) -> Self {
        Self {
            handle,
            owner_sessions,
        }
    }

    /// Resolves the effective owner for a register call: the explicit
    /// `owner` string, else the caller's unique bus name. Records the
    /// owner under the sender for disconnect cleanup.
    fn session_owner(&self, owner: &str, header: &Header<'_>) -> Owner {
        let sender = header
            .sender()
            .map(UniqueName::as_str)
            .unwrap_or_default()
            .to_string();
        let effective = if owner.is_empty() {
            sender.clone()
        } else {
            owner.to_string()
        };
        if !sender.is_empty()
            && let Ok(mut sessions) = self.owner_sessions.lock()
        {
            sessions
                .entry(sender)
                .or_default()
                .insert(effective.clone());
        }
        Owner::new(effective)
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
        self.handle.profiles()
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

    /// Active missing-pin completion configuration as a flat tuple
    /// (`radius_factor`, `max_missing`, `persist`, `mode`) - empty strings
    /// are impossible here; when nothing was published yet all values are
    /// defaults.
    async fn completion(&self) -> (f64, u32, bool, String) {
        let config = self.handle.snapshot().completion.unwrap_or_default();
        (
            config.radius_factor,
            config.max_missing,
            config.persist,
            config.mode.to_string(),
        )
    }

    /// Replaces the missing-pin completion configuration (`mode` is
    /// `match` or `exact`; `max_missing` must be 1 or 2).
    async fn set_completion(
        &self,
        radius_factor: f64,
        max_missing: u32,
        persist: bool,
        mode: &str,
    ) -> fdo::Result<()> {
        use tangible_objects_model::CompletionConfig;
        use tangible_objects_model::CompletionMode;

        let mode = mode.parse::<CompletionMode>().map_err(|_| {
            fdo::Error::InvalidArgs("mode must be one of: match, exact".to_string())
        })?;
        if !(1..=2).contains(&max_missing) {
            return Err(fdo::Error::InvalidArgs(
                "max_missing must be 1 or 2".to_string(),
            ));
        }
        let config = CompletionConfig {
            radius_factor,
            max_missing,
            persist,
            mode,
        };
        self.handle
            .send_command(ControlCommand::SetCompletion(config))
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Registers a figure profile at runtime: `points` is the flat pin
    /// list `[x0, y0, x1, y1, ...]` in mm. Queued to the device loop;
    /// fingerprint collisions are rejected there - verify via
    /// `list_profiles`.
    async fn register_pattern(
        &self,
        name: &str,
        points: Vec<f64>,
        tolerance: f64,
        owner: &str,
        persistent: bool,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        use tangible_objects_model::Millimeters;
        use tangible_objects_model::PatternTolerance;
        use tangible_objects_model::ProfileFile;
        use tangible_objects_model::Registration;
        use tangible_objects_model::Vec2;

        if !points.len().is_multiple_of(2) || points.is_empty() {
            return Err(fdo::Error::InvalidArgs(
                "points must be a non-empty even-length flat list".to_string(),
            ));
        }
        let points: Vec<Vec2<Millimeters>> = points
            .chunks_exact(2)
            .map(|pair| Vec2::new(pair[0], pair[1]))
            .collect();
        let profile = ProfileFile::from_points(name, &points, PatternTolerance::new(tolerance));
        self.handle
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::Pattern(profile)),
                owner: Some(self.session_owner(owner, &header)),
                lifetime: if persistent {
                    Lifetime::Persistent
                } else {
                    Lifetime::Session
                },
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Replaces a registered pattern's geometry while keeping its ID.
    async fn update_profile(
        &self,
        id: u32,
        name: &str,
        points: Vec<f64>,
        tolerance: f64,
    ) -> fdo::Result<()> {
        use tangible_objects_model::Millimeters;
        use tangible_objects_model::PatternId;
        use tangible_objects_model::PatternTolerance;
        use tangible_objects_model::ProfileFile;
        use tangible_objects_model::Vec2;

        if !points.len().is_multiple_of(2) || points.is_empty() {
            return Err(fdo::Error::InvalidArgs(
                "points must be a non-empty even-length flat list".to_string(),
            ));
        }
        let points: Vec<Vec2<Millimeters>> = points
            .chunks_exact(2)
            .map(|pair| Vec2::new(pair[0], pair[1]))
            .collect();
        let profile = ProfileFile::from_points(name, &points, PatternTolerance::new(tolerance));
        self.handle
            .send_command(ControlCommand::UpdateProfile {
                id: PatternId::new(id),
                profile: Box::new(profile),
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Removes a dynamically registered pattern by ID.
    async fn unregister_pattern(&self, id: u32) -> fdo::Result<()> {
        use tangible_objects_model::PatternId;

        self.handle
            .send_command(ControlCommand::UnregisterPattern {
                id: PatternId::new(id),
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Registers an occupancy field set from a TOML string (same shape as
    /// `--fields` files).
    async fn register_field_set(
        &self,
        toml_str: &str,
        owner: &str,
        persistent: bool,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        use tangible_objects_model::FieldSet;
        use tangible_objects_model::Registration;

        let set = toml::from_str::<FieldSet>(toml_str)
            .map_err(|e| fdo::Error::InvalidArgs(format!("invalid field set TOML: {e}")))?;
        self.handle
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::FieldSet(set)),
                owner: Some(self.session_owner(owner, &header)),
                lifetime: if persistent {
                    Lifetime::Persistent
                } else {
                    Lifetime::Session
                },
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Removes a field set by name.
    async fn unregister_field_set(&self, name: &str) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::UnregisterFieldSet {
                name: name.to_string(),
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// Removes every pattern and field set owned by `owner` - app
    /// cleanup hook; automatic cleanup also fires when a D-Bus caller's
    /// unique name vanishes.
    async fn unregister_owner(&self, owner: &str) -> fdo::Result<()> {
        self.handle
            .send_command(ControlCommand::UnregisterOwner {
                owner: Owner::new(owner),
            })
            .await
            .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
    }

    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    #[zbus(signal)]
    pub async fn object_moved(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A field became occupied (debounced placed transition).
    #[zbus(signal)]
    pub async fn field_placed(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
    ) -> zbus::Result<()>;

    /// A field became empty (debounced lifted transition).
    #[zbus(signal)]
    pub async fn field_lifted(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
    ) -> zbus::Result<()>;

    /// A field's debounced contact count changed.
    #[zbus(signal)]
    pub async fn field_count_changed(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
        count: u32,
    ) -> zbus::Result<()>;
}
