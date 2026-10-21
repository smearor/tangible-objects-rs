#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// Discriminator for [`crate::domain::TangibleEvent`] payloads.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
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
// Under `cynic` the derive owns serde - the wire form is the same
// SCREAMING_SNAKE either way, so the feature switch is invisible on
// the wire.
pub enum EventKind {
    /// The set of active touch points changed.
    TouchPointsUpdated,
    /// A new tangible was detected and is now tracked.
    TangibleAdded,
    /// A tracked tangible's pose changed.
    TangibleMoved,
    /// A tracked tangible expired.
    TangibleRemoved,
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
    /// An occupancy field became occupied (debounced placed).
    FieldPlaced,
    /// An occupancy field became empty (debounced lifted).
    FieldLifted,
    /// An occupancy field's debounced contact count changed.
    FieldCountChanged,
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
    /// A recording session stopped (`count` carries the frames written).
    RecordingStopped,
    /// A file-triggered config reload took effect.
    ConfigReloaded,
    /// Events were skipped because this subscription lagged - a control
    /// event, never requestable via `kinds`; `skipped` carries the count.
    Lagged,
}
