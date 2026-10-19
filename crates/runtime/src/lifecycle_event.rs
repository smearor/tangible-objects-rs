use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::StageName;

/// Lifecycle event emitted when the device loop *applies* a control
/// command - not frame-derived like [`EngineEvent`] but a session-level
/// fact: what actually took effect.
///
/// Subscribers see the applied outcome only: a rejected registration or an
/// ignored command produces no event (SUBSCRIPTIONS.md).
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// A pattern was registered (`Register`) or replaced (`UpdateProfile`).
    ProfileRegistered {
        /// Pattern ID under which it was registered.
        pattern_id: PatternId,
        /// Registered figure name.
        name: PatternName,
        /// Owning application, when bound to one.
        owner: Option<Owner>,
    },
    /// A registered pattern was removed (unregister or owner cleanup).
    ProfileUnregistered {
        /// Removed pattern ID.
        pattern_id: PatternId,
    },
    /// A field set was registered.
    FieldSetRegistered {
        /// Registered field-set name.
        name: FieldSetName,
        /// Owning application, when bound to one.
        owner: Option<Owner>,
    },
    /// A field set was removed (unregister or owner cleanup).
    FieldSetUnregistered {
        /// Removed field-set name.
        name: FieldSetName,
    },
    /// A scoped tangible stage was created on the pipeline - the first
    /// `stage = "name"` profile registered (EVENT_PIPELINE.md phase 6).
    StageRegistered {
        /// New stage's scope name.
        name: StageName,
    },
    /// A scoped tangible stage was removed - its last pattern went
    /// away; its suppression state died with it.
    StageUnregistered {
        /// Removed stage's scope name.
        name: StageName,
    },
    /// The completion configuration changed (`SetCompletion` applied).
    CompletionChanged(CompletionConfig),
    /// The filter-proxy suppression policy changed (`SetFilterMode`
    /// applied).
    FilterModeChanged(FilterMode),
    /// A recording session started (`StartRecording` applied).
    RecordingStarted,
    /// A recording session stopped (`StopRecording` applied); `frames` is
    /// the number of frames written to the `.jsonl` artifact.
    RecordingStopped {
        /// Frames recorded during the session.
        frames: FrameCount,
    },
    /// A file-triggered configuration reload took effect - a generic
    /// "refresh your view" marker for remote clients. Reserved: no
    /// file-watch path emits it yet.
    ConfigReloaded,
}
