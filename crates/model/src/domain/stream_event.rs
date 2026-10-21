use serde::Deserialize;
use serde::Serialize;

use super::event_kind::EventKind;

/// Event kind a client may request via `eventStream(kinds: ...)`.
///
/// Mirrors [`EventKind`] minus `LAGGED` - the lag control event is never
/// requestable, it is emitted whenever this subscription drops events
/// regardless of the `kinds` filter (SUBSCRIPTIONS.md).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
pub enum StreamEvent {
    /// A new tangible was detected and is now tracked.
    TangibleAdded,
    /// A tracked tangible's pose changed.
    TangibleMoved,
    /// A tracked tangible expired.
    TangibleRemoved,
    /// The set of active touch points changed - high-rate (~120 Hz),
    /// respect `maxHz`.
    TouchPointsUpdated,
    /// An occupancy field became occupied.
    FieldPlaced,
    /// An occupancy field became empty.
    FieldLifted,
    /// An occupancy field's debounced contact count changed.
    FieldCountChanged,
    /// All mandatory set members became present (TOS-ADD).
    SetAdded,
    /// The last mandatory set member disappeared (TOS-REM).
    SetRemoved,
    /// A member tangible appeared in an active set (TOS-OBJ-ADD).
    ObjectAdded,
    /// A member tangible disappeared (TOS-OBJ-REM).
    ObjectRemoved,
    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    ObjectMoved,
    /// A pattern was registered or replaced at runtime.
    ProfileRegistered,
    /// A registered pattern was removed.
    ProfileUnregistered,
    /// A field set was registered at runtime.
    FieldSetRegistered,
    /// A field set was removed.
    FieldSetUnregistered,
    /// A scoped tangible stage was created on the pipeline.
    StageRegistered,
    /// A scoped tangible stage was removed.
    StageUnregistered,
    /// The completion configuration changed.
    CompletionChanged,
    /// The filter-proxy suppression policy changed.
    FilterModeChanged,
    /// A recording session started.
    RecordingStarted,
    /// A recording session stopped.
    RecordingStopped,
    /// A file-triggered config reload took effect.
    ConfigReloaded,
}

impl StreamEvent {
    /// The [`EventKind`] emitted for this request kind - a 1:1 mapping;
    /// request and emitted kinds share the same vocabulary.
    pub fn kind(self) -> EventKind {
        match self {
            Self::TangibleAdded => EventKind::TangibleAdded,
            Self::TangibleMoved => EventKind::TangibleMoved,
            Self::TangibleRemoved => EventKind::TangibleRemoved,
            Self::TouchPointsUpdated => EventKind::TouchPointsUpdated,
            Self::FieldPlaced => EventKind::FieldPlaced,
            Self::FieldLifted => EventKind::FieldLifted,
            Self::FieldCountChanged => EventKind::FieldCountChanged,
            Self::SetAdded => EventKind::SetAdded,
            Self::SetRemoved => EventKind::SetRemoved,
            Self::ObjectAdded => EventKind::ObjectAdded,
            Self::ObjectRemoved => EventKind::ObjectRemoved,
            Self::ObjectMoved => EventKind::ObjectMoved,
            Self::ProfileRegistered => EventKind::ProfileRegistered,
            Self::ProfileUnregistered => EventKind::ProfileUnregistered,
            Self::FieldSetRegistered => EventKind::FieldSetRegistered,
            Self::StageRegistered => EventKind::StageRegistered,
            Self::StageUnregistered => EventKind::StageUnregistered,
            Self::FieldSetUnregistered => EventKind::FieldSetUnregistered,
            Self::CompletionChanged => EventKind::CompletionChanged,
            Self::FilterModeChanged => EventKind::FilterModeChanged,
            Self::RecordingStarted => EventKind::RecordingStarted,
            Self::RecordingStopped => EventKind::RecordingStopped,
            Self::ConfigReloaded => EventKind::ConfigReloaded,
        }
    }
}
