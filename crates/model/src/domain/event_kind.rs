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
    /// A seat was added to a seat session.
    SeatAdded,
    /// An anchored seat's derived pose changed (marker moved).
    SeatMoved,
    /// A seat was removed from its session.
    SeatRemoved,
    /// A marker tangible bound to a seat (first bind or rebind).
    SeatMarkerBound,
    /// A seat's marker tangible stopped being tracked - the seat
    /// keeps its last pose.
    SeatMarkerAbsent,
    /// A seat session was claimed by an application.
    SessionClaimed,
    /// A seat session was released (explicit or owner cleanup).
    SessionReleased,
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
    /// A dead zone was registered at runtime.
    DeadZoneRegistered,
    /// A dead zone was removed.
    DeadZoneUnregistered,
    /// A spell was registered.
    SpellRegistered,
    /// A spell was removed.
    SpellUnregistered,
    /// A gesture matched a spell.
    SpellMatched,
    /// A handprint was registered.
    HandProfileRegistered,
    /// A handprint was removed.
    HandProfileUnregistered,
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

/// `snake_case` text name of the kind - the wire form stays
/// `SCREAMING_SNAKE_CASE` (serde/cynic); `Display` is what log lines and
/// event descriptions print.
impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::TouchPointsUpdated => "touch_points_updated",
            Self::TangibleAdded => "tangible_added",
            Self::TangibleMoved => "tangible_moved",
            Self::TangibleRemoved => "tangible_removed",
            Self::SetAdded => "set_added",
            Self::SetRemoved => "set_removed",
            Self::ObjectAdded => "object_added",
            Self::ObjectRemoved => "object_removed",
            Self::ObjectMoved => "object_moved",
            Self::SeatAdded => "seat_added",
            Self::SeatMoved => "seat_moved",
            Self::SeatRemoved => "seat_removed",
            Self::SeatMarkerBound => "seat_marker_bound",
            Self::SeatMarkerAbsent => "seat_marker_absent",
            Self::SessionClaimed => "session_claimed",
            Self::SessionReleased => "session_released",
            Self::FieldPlaced => "field_placed",
            Self::FieldLifted => "field_lifted",
            Self::FieldCountChanged => "field_count_changed",
            Self::ProfileRegistered => "profile_registered",
            Self::ProfileUnregistered => "profile_unregistered",
            Self::FieldSetRegistered => "field_set_registered",
            Self::FieldSetUnregistered => "field_set_unregistered",
            Self::DeadZoneRegistered => "dead_zone_registered",
            Self::DeadZoneUnregistered => "dead_zone_unregistered",
            Self::SpellRegistered => "spell_registered",
            Self::SpellUnregistered => "spell_unregistered",
            Self::SpellMatched => "spell_matched",
            Self::HandProfileRegistered => "hand_profile_registered",
            Self::HandProfileUnregistered => "hand_profile_unregistered",
            Self::StageRegistered => "stage_registered",
            Self::StageUnregistered => "stage_unregistered",
            Self::CompletionChanged => "completion_changed",
            Self::FilterModeChanged => "filter_mode_changed",
            Self::RecordingStarted => "recording_started",
            Self::RecordingStopped => "recording_stopped",
            Self::ConfigReloaded => "config_reloaded",
            Self::Lagged => "lagged",
        };
        f.write_str(name)
    }
}
