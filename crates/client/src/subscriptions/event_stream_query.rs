use serde::Serialize;
use serde_json::Value;
use serde_json::to_value;
use tangible_objects_model::PatternId;
use tangible_objects_model::StreamEvent;
use tangible_objects_model::TangibleSetId;

/// Filter for the `eventStream` subscription - mirrors the schema
/// arguments; an empty `kinds` list is rejected by the server.
#[derive(Debug, Default, Clone)]
pub struct EventStreamQuery {
    /// Event kinds to receive (`SUBSCRIPTIONS.md` vocabulary).
    pub kinds: Vec<StreamEvent>,
    /// Narrow to events about this pattern.
    pub pattern_id: Option<PatternId>,
    /// Narrow to events about this set.
    pub set_id: Option<TangibleSetId>,
    /// Throttle `TOUCH_POINTS_UPDATED` to at most this many events/sec.
    pub max_hz: Option<u32>,
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

    /// Serializes into the `eventStream` variables shape
    /// (`kinds` as SCREAMING enum strings).
    pub(crate) fn to_variables(&self) -> Value {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Vars {
            kinds: Vec<String>,
            pattern_id: Option<u32>,
            set_id: Option<u32>,
            max_hz: Option<u32>,
        }
        let vars = Vars {
            kinds: self
                .kinds
                .iter()
                .map(|k| {
                    to_value(k)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default()
                })
                .collect(),
            pattern_id: self.pattern_id.map(|p| p.as_u32()),
            set_id: self.set_id.map(|s| s.as_u32()),
            max_hz: self.max_hz,
        };
        to_value(vars).unwrap_or_default()
    }
}
