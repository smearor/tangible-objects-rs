#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;
use std::str::FromStr;

use super::StreamEventParseError;
use super::event_kind::EventKind;

/// Event kind a client may request via `eventStream(kinds: ...)`.
///
/// Mirrors [`EventKind`] minus `LAGGED` - the lag control event is never
/// requestable, it is emitted whenever this subscription drops events
/// regardless of the `kinds` filter (SUBSCRIPTIONS.md).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(not(feature = "cynic"), serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
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

impl FromStr for StreamEvent {
    type Err = StreamEventParseError;

    /// Parses a request kind - case-insensitive `SCREAMING_SNAKE` wire
    /// names (`"tangible_added"`, `"TANGIBLE_ADDED"` both work; the CLI's
    /// comma-separated `--kinds` values).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TANGIBLE_ADDED" => Ok(Self::TangibleAdded),
            "TANGIBLE_MOVED" => Ok(Self::TangibleMoved),
            "TANGIBLE_REMOVED" => Ok(Self::TangibleRemoved),
            "TOUCH_POINTS_UPDATED" => Ok(Self::TouchPointsUpdated),
            "FIELD_PLACED" => Ok(Self::FieldPlaced),
            "FIELD_LIFTED" => Ok(Self::FieldLifted),
            "FIELD_COUNT_CHANGED" => Ok(Self::FieldCountChanged),
            "SET_ADDED" => Ok(Self::SetAdded),
            "SET_REMOVED" => Ok(Self::SetRemoved),
            "OBJECT_ADDED" => Ok(Self::ObjectAdded),
            "OBJECT_REMOVED" => Ok(Self::ObjectRemoved),
            "OBJECT_MOVED" => Ok(Self::ObjectMoved),
            "PROFILE_REGISTERED" => Ok(Self::ProfileRegistered),
            "PROFILE_UNREGISTERED" => Ok(Self::ProfileUnregistered),
            "FIELD_SET_REGISTERED" => Ok(Self::FieldSetRegistered),
            "FIELD_SET_UNREGISTERED" => Ok(Self::FieldSetUnregistered),
            "STAGE_REGISTERED" => Ok(Self::StageRegistered),
            "STAGE_UNREGISTERED" => Ok(Self::StageUnregistered),
            "COMPLETION_CHANGED" => Ok(Self::CompletionChanged),
            "FILTER_MODE_CHANGED" => Ok(Self::FilterModeChanged),
            "RECORDING_STARTED" => Ok(Self::RecordingStarted),
            "RECORDING_STOPPED" => Ok(Self::RecordingStopped),
            "CONFIG_RELOADED" => Ok(Self::ConfigReloaded),
            _ => Err(StreamEventParseError::new(s)),
        }
    }
}
