use std::path::PathBuf;

use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;
use tangible_objects_model::StageName;

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
        /// Scope of the stage to reconfigure (EVENT_PIPELINE.md phase 7):
        /// `Some` applies the policy to that stage only, `None`
        /// broadcasts to all tangible stages. Positional stages always
        /// ignore `SetFilterMode`.
        stage: Option<StageName>,
    },
    /// Create or claim a pipeline stage for `name` before any `stage =
    /// "name"` pattern registers - the stage persists until
    /// `UnregisterStage`, independent of pattern lifetimes
    /// (EVENT_PIPELINE.md phase 7). Lets a client pre-configure a
    /// stage's `SetFilterMode` policy.
    RegisterStage {
        /// Stage scope name.
        name: StageName,
    },
    /// Remove the pipeline stage for `name` - its suppression state
    /// dies with it; patterns of that scope keep matching but are no
    /// longer stage-suppressed until a new stage exists.
    UnregisterStage {
        /// Stage scope name.
        name: StageName,
    },
    /// Capture a new figure profile with the given name.
    TrainProfile(PatternName),
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
    /// Reset the accumulated [`crate::SessionMetrics`] counters - e.g.
    /// between automated test runs.
    ResetMetrics,
    /// Display a user-facing message in the GUI ("Legen Sie Figur 10
    /// auf") - interfaces steer the operator during test series.
    /// `None`-equivalent empty string clears it.
    ShowMessage(String),
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
        name: FieldSetName,
    },
    /// Remove every pattern and field set owned by `owner` - used for
    /// explicit app cleanup and D-Bus disconnect handling.
    UnregisterOwner {
        /// Owner name (or the caller's unique bus name on D-Bus).
        owner: Owner,
    },
    /// Show or hide the daemon-embedded GUI window (`crates/gtk`,
    /// `--gui` mode). Part of the command vocabulary on all builds -
    /// daemons without the `gui` feature ignore it (the runtime's
    /// `gui_visible` state is feature-gated).
    SetGuiVisible(bool),
}
