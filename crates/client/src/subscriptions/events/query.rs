use cynic::QueryVariables;
use tangible_objects_model::PatternId;
use tangible_objects_model::StreamEvent;
use tangible_objects_model::TangibleSetId;

/// Filter for the `eventStream` subscription - mirrors the schema
/// arguments; an empty `kinds` list is rejected by the server.
/// Serialized by cynic as the operation's variables.
#[derive(Debug, Default, Clone, QueryVariables)]
#[cynic(
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub struct EventStreamQuery {
    /// Event kinds to receive (`SUBSCRIPTIONS.md` vocabulary).
    pub kinds: Vec<StreamEvent>,
    /// Narrow to events about this pattern.
    pub pattern_id: Option<PatternId>,
    /// Narrow to events about this set.
    pub set_id: Option<TangibleSetId>,
    /// Throttle `TOUCH_POINTS_UPDATED` to at most this many events/sec.
    pub max_hz: Option<i32>,
}

impl EventStreamQuery {
    /// Every requestable kind, unthrottled and unfiltered - the
    /// "give me everything" subscription (the server rejects an
    /// empty `kinds` list, so every variant is enumerated here).
    pub fn all() -> Self {
        Self {
            kinds: vec![
                StreamEvent::TangibleAdded,
                StreamEvent::TangibleMoved,
                StreamEvent::TangibleRemoved,
                StreamEvent::TouchPointsUpdated,
                StreamEvent::FieldPlaced,
                StreamEvent::FieldLifted,
                StreamEvent::FieldCountChanged,
                StreamEvent::SetAdded,
                StreamEvent::SetRemoved,
                StreamEvent::ObjectAdded,
                StreamEvent::ObjectRemoved,
                StreamEvent::ObjectMoved,
                StreamEvent::ProfileRegistered,
                StreamEvent::ProfileUnregistered,
                StreamEvent::FieldSetRegistered,
                StreamEvent::FieldSetUnregistered,
                StreamEvent::StageRegistered,
                StreamEvent::StageUnregistered,
                StreamEvent::CompletionChanged,
                StreamEvent::FilterModeChanged,
                StreamEvent::RecordingStarted,
                StreamEvent::RecordingStopped,
                StreamEvent::ConfigReloaded,
            ],
            pattern_id: None,
            set_id: None,
            max_hz: None,
        }
    }
}
