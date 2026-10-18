use std::path::PathBuf;

use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;

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
    /// Replace the missing-pin completion configuration at runtime
    /// (TRACKING_STABILIZATION.md). Applies to matching and suppression in
    /// all modes from the next frame on.
    SetCompletion(CompletionConfig),
    /// Register a profile or field set at runtime (DYNAMIC_PROFILES.md).
    /// Registrations whose fingerprint collides with an already-loaded
    /// profile are rejected in the device loop.
    Register {
        /// What to register - boxed: `ProfileFile` makes this variant
        /// large; boxing keeps `ControlCommand` small for the channel.
        registration: Box<Registration>,
        /// Optional application name the registration belongs to -
        /// `UnregisterOwner` removes every registration of an owner;
        /// D-Bus binds registrations to the caller's unique name for
        /// automatic disconnect cleanup.
        owner: Option<Owner>,
        /// `Session` (default) or `Persistent` - persistent registrations
        /// are written to the daemon's runtime store on apply.
        lifetime: Lifetime,
    },
    /// Replace a registered pattern's geometry/name while keeping its
    /// ID (`UpdateProfile`). Applies to new matches; tracked tangibles
    /// keep their recorded pose.
    UpdateProfile {
        /// Pattern to replace.
        id: PatternId,
        /// New profile definition - boxed like `Register`.
        profile: Box<ProfileFile>,
    },
    /// Remove a dynamically registered pattern by its pattern ID. The ID
    /// is never reused.
    UnregisterPattern {
        /// Pattern to remove.
        id: PatternId,
    },
    /// Remove a field set by name.
    UnregisterFieldSet {
        /// Field set to remove.
        name: String,
    },
    /// Remove every pattern and field set owned by `owner` - used for
    /// explicit app cleanup and D-Bus disconnect handling.
    UnregisterOwner {
        /// Owner name (or the caller's unique bus name on D-Bus).
        owner: Owner,
    },
}
