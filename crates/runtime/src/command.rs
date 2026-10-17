use std::path::PathBuf;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RecordLabel;

/// Control command sent by an interface back to the daemon's device loop.
///
/// Commands are drained by the runtime after each published frame
/// ([`crate::EngineRuntime::try_recv_command`]); the concrete behavior is
/// implemented where the owning functionality lands (e.g. filter mode in
/// Phase 4, remote training with the interface crates).
#[derive(Debug, Clone)]
pub enum ControlCommand {
    /// Switch the filter proxy's suppression policy
    /// ([`FilterMode::Off`] = transparent passthrough). `stability`
    /// overrides the consecutive-matched-frames gate when `Some`; `None`
    /// keeps the current value.
    SetFilterMode {
        /// New suppression policy.
        mode: FilterMode,
        /// Consecutive frames a slot must match before suppression starts.
        stability: Option<FrameCount>,
    },
    /// Capture a new figure profile with the given name.
    TrainProfile(String),
    /// Start a recording session writing `<base_path>.jsonl` (frames) and
    /// `<base_path>.evtest` (raw events).
    StartRecording {
        /// Base path of the recording artifacts, without extension.
        base_path: PathBuf,
        /// Ground-truth label applied to recorded frames until changed.
        label: Option<RecordLabel>,
    },
    /// Stop the active recording session.
    StopRecording,
    /// Change the ground-truth label applied to subsequent recorded frames.
    SetRecordLabel(Option<RecordLabel>),
}
