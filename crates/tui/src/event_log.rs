//! `EventLog` - ring buffer of the lifecycle/set/field events the
//! stream delivered, rendered by the event-log pane (TUI.md phase 4).

use std::collections::VecDeque;

use crate::i18n::language_loader;
use i18n_embed_fl::fl;
use tangible_objects_model::EventKind;
use tangible_objects_model::TangibleEvent;

/// Lines the log keeps - scrolling history beyond the visible pane
/// lets an operator re-check what just happened.
const LOG_CAPACITY: usize = 200;

/// Ring buffer of rendered log lines.
pub struct EventLog {
    /// Buffered lines, newest last.
    lines: VecDeque<String>,
}

impl EventLog {
    /// Creates an empty log.
    pub fn new() -> Self {
        Self {
            lines: VecDeque::with_capacity(LOG_CAPACITY + 1),
        }
    }

    /// Appends `event`'s rendered description and drops the oldest
    /// line when the buffer overflows.
    pub fn record(&mut self, event: &TangibleEvent) {
        if let Some(line) = describe(event) {
            if self.lines.len() == LOG_CAPACITY {
                self.lines.pop_front();
            }
            self.lines.push_back(line);
        }
    }

    /// All buffered lines, oldest first.
    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }
}

/// One-line description of an event payload for the log pane; touch
/// and tangible streams are excluded (too noisy - the canvas shows
/// them continuously).
fn describe(event: &TangibleEvent) -> Option<String> {
    let name = event.name.as_deref().unwrap_or_default();
    let detail = event.detail.as_deref().unwrap_or_default();
    let line = match event.kind {
        EventKind::TouchPointsUpdated
        | EventKind::TangibleAdded
        | EventKind::TangibleMoved
        | EventKind::TangibleRemoved => return None,
        EventKind::SetAdded | EventKind::SetRemoved => {
            format!("{} set {}", event.kind, event.set_id?.as_u32())
        }
        EventKind::ObjectAdded | EventKind::ObjectRemoved | EventKind::ObjectMoved => {
            format!(
                "{} set {} tang {}",
                event.kind,
                event.set_id?.as_u32(),
                event
                    .tangible
                    .as_ref()
                    .map(|t| t.pattern_name.as_str())
                    .unwrap_or_default()
            )
        }
        EventKind::FieldPlaced | EventKind::FieldLifted => format!(
            "{} {} {}",
            event.kind,
            event
                .field_set
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
            event.field.as_ref().map(|f| f.as_str()).unwrap_or_default()
        ),
        EventKind::FieldCountChanged => format!(
            "{} {} {} = {}",
            event.kind,
            event
                .field_set
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
            event.field.as_ref().map(|f| f.as_str()).unwrap_or_default(),
            event.count?.as_u32()
        ),
        EventKind::ProfileRegistered => format!(
            "{} {name} id{} {}",
            event.kind,
            event.pattern_id?.as_u32(),
            event.owner.as_ref().map(|o| o.as_str()).unwrap_or_default()
        ),
        EventKind::ProfileUnregistered => {
            format!("{} id{}", event.kind, event.pattern_id?.as_u32())
        }
        EventKind::FieldSetRegistered
        | EventKind::FieldSetUnregistered
        | EventKind::DeadZoneRegistered
        | EventKind::DeadZoneUnregistered
        | EventKind::StageRegistered
        | EventKind::StageUnregistered => format!("{} {name}", event.kind),
        EventKind::CompletionChanged | EventKind::FilterModeChanged => {
            format!("{} {detail}", event.kind)
        }
        EventKind::RecordingStarted => event.kind.to_string(),
        EventKind::RecordingStopped => {
            format!("{} {} frames", event.kind, event.frames?.as_u64())
        }
        EventKind::ConfigReloaded => event.kind.to_string(),
        EventKind::Lagged => fl!(
            language_loader(),
            "log-lagged",
            count = (event.skipped.map(|s| s.as_u64()).unwrap_or_default() as i64)
        ),
    };
    Some(line)
}

#[cfg(test)]
mod tests {
    use super::EventLog;
    use tangible_objects_model::Owner;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::TangibleEvent;

    /// Noisy frame events never enter the log.
    #[test]
    fn touch_events_are_not_logged() {
        let mut log = EventLog::new();
        let event = TangibleEvent::from(&tangible_objects_model::DomainEvent::Engine(
            tangible_objects_model::EngineEvent::TouchPointsUpdated,
        ));
        log.record(&event);
        assert!(log.lines().next().is_none());
    }

    /// A lifecycle event renders its kind name plus payload fields.
    #[test]
    fn profile_registration_line() {
        let mut log = EventLog::new();
        let mut event = TangibleEvent::from(&tangible_objects_model::DomainEvent::Engine(
            tangible_objects_model::EngineEvent::TouchPointsUpdated,
        ));
        event.kind = tangible_objects_model::EventKind::ProfileRegistered;
        event.pattern_id = Some(PatternId::new(3));
        event.name = Some("fig_3pins".to_string());
        event.owner = Some(Owner::new("app"));
        log.record(&event);
        let line = log.lines().next().cloned().unwrap_or_default();
        assert!(line.contains("profile_registered"), "line: {line}");
        assert!(line.contains("fig_3pins"), "line: {line}");
    }

    /// The buffer drops the oldest line past capacity.
    #[test]
    fn ring_buffer_drops_oldest() {
        let mut log = EventLog::new();
        let mut event = TangibleEvent::from(&tangible_objects_model::DomainEvent::Engine(
            tangible_objects_model::EngineEvent::TouchPointsUpdated,
        ));
        event.kind = tangible_objects_model::EventKind::StageRegistered;
        for index in 0..210 {
            event.name = Some(format!("stage_{index}"));
            log.record(&event);
        }
        assert_eq!(log.lines().count(), 200);
        assert_eq!(
            log.lines().next().map(String::as_str),
            Some("stage_registered stage_10")
        );
    }
}
