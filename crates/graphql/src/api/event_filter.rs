use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use tangible_objects_model::FieldSetName;
use tangible_objects_model::PatternId;
use tangible_objects_model::TangibleSetId;

use super::EventKind;
use super::StreamEvent;
use super::TangibleEvent;

/// Default `maxHz` for the throttled `TOUCH_POINTS_UPDATED` kind
/// (SUBSCRIPTIONS.md: ~30 Hz).
pub(crate) const DEFAULT_MAX_HZ: u32 = 30;

/// Server-side filter for `eventStream`/`tangibleEvents`/`fieldEvents`:
/// which kinds pass, plus optional `patternId`/`setId`/`fieldSet` narrows
/// and the `TOUCH_POINTS_UPDATED` throttle.
pub(crate) struct EventFilter {
    /// Event kinds the subscriber requested - anything else is dropped.
    kinds: HashSet<EventKind>,
    /// Optional narrow: only events about this pattern pass
    /// (`TANGIBLE_*`/`OBJECT_*`/`PROFILE_*` kinds).
    pattern_id: Option<PatternId>,
    /// Optional narrow: only events about this tangible set pass
    /// (`SET_*`/`OBJECT_*` kinds).
    set_id: Option<TangibleSetId>,
    /// Optional narrow: only events about this field set pass
    /// (`FIELD_*` kinds).
    field_set: Option<FieldSetName>,
    /// Minimum interval between emitted `TOUCH_POINTS_UPDATED` events.
    touch_interval: Option<Duration>,
    /// When the last `TOUCH_POINTS_UPDATED` passed the throttle.
    last_touch_emit: Option<Instant>,
}

impl EventFilter {
    /// Builds the filter for `eventStream`: `kinds` is the client's
    /// request (non-empty - validated by the resolver), the options narrow
    /// the stream further and `max_hz` throttles the touch kind.
    pub(crate) fn new(
        kinds: &[StreamEvent],
        pattern_id: Option<PatternId>,
        set_id: Option<TangibleSetId>,
        field_set: Option<FieldSetName>,
        max_hz: Option<u32>,
    ) -> Self {
        Self {
            kinds: kinds.iter().map(|kind| kind.kind()).collect(),
            pattern_id,
            set_id,
            field_set,
            touch_interval: Some(Duration::from_millis(
                1000 / u64::from(max_hz.unwrap_or(DEFAULT_MAX_HZ).max(1)),
            )),
            last_touch_emit: None,
        }
    }

    /// Whether `event` passes the kind, id and throttle checks. `LAGGED`
    /// control events always pass - they must reach the client regardless
    /// of the requested kinds.
    pub(crate) fn accepts(&mut self, event: &TangibleEvent) -> bool {
        if event.kind == EventKind::Lagged {
            return true;
        }
        if !self.kinds.contains(&event.kind) {
            return false;
        }
        if let Some(pattern_id) = self.pattern_id
            && event
                .tangible
                .as_ref()
                .is_none_or(|tangible| tangible.pattern_id != pattern_id)
        {
            return false;
        }
        if let Some(set_id) = self.set_id
            && event.set_id != Some(set_id)
        {
            return false;
        }
        if let Some(field_set) = &self.field_set
            && event.field_set.as_ref() != Some(field_set)
        {
            return false;
        }
        if event.kind == EventKind::TouchPointsUpdated
            && let Some(interval) = self.touch_interval
        {
            let now = Instant::now();
            if self
                .last_touch_emit
                .is_some_and(|last| now.duration_since(last) < interval)
            {
                return false;
            }
            self.last_touch_emit = Some(now);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::EventFilter;
    use crate::EventKind;
    use crate::StreamEvent;
    use crate::TangibleEvent;
    use tangible_objects_model::PatternId;

    fn event(kind: EventKind) -> TangibleEvent {
        TangibleEvent {
            kind,
            tangible: None,
            set_id: None,
            field_set: None,
            field: None,
            count: None,
            frames: None,
            pattern_id: None,
            name: None,
            owner: None,
            detail: None,
            skipped: None,
        }
    }

    #[test]
    fn only_requested_kinds_pass() {
        let mut filter = EventFilter::new(&[StreamEvent::TangibleAdded], None, None, None, None);
        assert!(filter.accepts(&event(EventKind::TangibleAdded)));
        assert!(!filter.accepts(&event(EventKind::TangibleMoved)));
        assert!(!filter.accepts(&event(EventKind::FieldPlaced)));
        assert!(!filter.accepts(&event(EventKind::ProfileRegistered)));
    }

    #[test]
    fn lagged_always_passes() {
        let mut filter = EventFilter::new(&[StreamEvent::TangibleAdded], None, None, None, None);
        assert!(filter.accepts(&event(EventKind::Lagged)));
    }

    #[test]
    fn pattern_id_narrows_tangible_events() {
        let mut filter = EventFilter::new(
            &[StreamEvent::TangibleAdded],
            Some(PatternId::new(5)),
            None,
            None,
            None,
        );
        let mut added = event(EventKind::TangibleAdded);
        added.tangible = Some(tangible_objects_model::TrackedTangible {
            id: tangible_objects_model::TangibleId::new(1),
            pattern_id: PatternId::new(5),
            pattern_name: "fig".into(),
            center: tangible_objects_model::Vec2::new(0.0, 0.0),
            rotation_rad: tangible_objects_model::RotationRad::new(0.0),
            pending: false,
            pending_frames: tangible_objects_model::FrameCount::new(0),
            spawn_stability: tangible_objects_model::DEFAULT_SPAWN_STABILITY,
            frames_since_match: tangible_objects_model::FrameCount::new(0),
        });
        assert!(filter.accepts(&added));
        added.tangible.as_mut().unwrap().pattern_id = PatternId::new(9);
        assert!(!filter.accepts(&added));
        // an event without a tangible never matches a patternId filter
        assert!(!filter.accepts(&event(EventKind::ProfileRegistered)));
    }

    #[test]
    fn touch_updates_are_throttled() {
        let mut filter = EventFilter::new(
            &[StreamEvent::TouchPointsUpdated],
            None,
            None,
            None,
            Some(30),
        );
        assert!(filter.accepts(&event(EventKind::TouchPointsUpdated)));
        // immediately repeated - inside the 33 ms interval
        assert!(!filter.accepts(&event(EventKind::TouchPointsUpdated)));
    }

    #[test]
    fn field_set_name_narrows_field_events() {
        let mut filter = EventFilter::new(
            &[StreamEvent::FieldPlaced],
            None,
            None,
            Some("board".into()),
            None,
        );
        let mut placed = event(EventKind::FieldPlaced);
        placed.field_set = Some("board".into());
        assert!(filter.accepts(&placed));
        placed.field_set = Some("other".into());
        assert!(!filter.accepts(&placed));
    }
}
