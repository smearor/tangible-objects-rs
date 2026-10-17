use tangible_objects_model::Calibration;
use tangible_objects_model::CandidateReport;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tracing::debug;

use super::EngineEvent;
use super::EngineFrame;
use super::TouchState;
use super::Tracker;
use super::matching;

/// Default EMA smoothing factor for tracked poses.
pub const DEFAULT_EMA_ALPHA: f64 = 0.4;

/// Default association radius between a match and a tracked tangible.
pub const DEFAULT_MAX_CENTER_DISTANCE: Millimeters = Millimeters::new(200.0);

/// Default number of frames a tangible survives without a match.
pub const DEFAULT_MAX_FRAMES_WITHOUT_MATCH: u32 = 30;

/// Headless detection engine: slot state machine + pattern matching +
/// frame-to-frame tracking.
///
/// Feed [`RawEvent`]s via [`Engine::process`]; every `SYN_REPORT` produces an
/// [`EngineFrame`] with the frame's matches, touches and domain events, and
/// updates the tracker.
pub struct Engine {
    touch_state: TouchState,
    patterns: Vec<TangiblePattern>,
    tracker: Tracker,
    calibration: Option<Calibration>,
    prev_touches: Vec<TouchPoint>,
}

impl Engine {
    /// Creates an engine with the given figure patterns and default
    /// tracker parameters.
    pub fn new(patterns: Vec<TangiblePattern>) -> Self {
        Self {
            touch_state: TouchState::new(),
            patterns,
            tracker: Tracker::new(
                DEFAULT_EMA_ALPHA,
                DEFAULT_MAX_CENTER_DISTANCE,
                DEFAULT_MAX_FRAMES_WITHOUT_MATCH,
            ),
            calibration: None,
            prev_touches: Vec::new(),
        }
    }

    /// Sets the device calibration used for raw-to-mm conversion.
    pub fn set_calibration(&mut self, calibration: Calibration) {
        self.calibration = Some(calibration);
        self.touch_state.set_calibration(calibration);
    }

    /// Currently configured calibration, if any.
    pub fn calibration(&self) -> Option<Calibration> {
        self.calibration
    }

    /// The loaded figure patterns.
    pub fn patterns(&self) -> &[TangiblePattern] {
        &self.patterns
    }

    /// All currently active touch points.
    pub fn touches(&self) -> Vec<TouchPoint> {
        self.touch_state.active_touches()
    }

    /// Number of currently active touch points.
    pub fn touch_count(&self) -> usize {
        self.touch_state.active_count()
    }

    /// All currently tracked tangibles (EMA-smoothed).
    pub fn tracked(&self) -> &[TrackedTangible] {
        self.tracker.active()
    }

    /// Mutable access to the tracker (e.g. to clear it on mode switch).
    pub fn tracker_mut(&mut self) -> &mut Tracker {
        &mut self.tracker
    }

    /// Finds all pattern matches in a given set of touch points without
    /// touching engine state.
    pub fn find_matches(&self, touches: &[TouchPoint]) -> Vec<PatternMatch> {
        matching::find_matches(touches, &self.patterns)
    }

    /// Feeds one raw event into the engine.
    ///
    /// Returns `Some` with the frame's matches, touches and domain events on
    /// each `SYN_REPORT`, `None` for all other events.
    ///
    /// This is the production path: no candidate diagnostics are emitted and
    /// [`EngineFrame::candidates`] stays empty.
    pub fn process(&mut self, event: &RawEvent) -> Option<EngineFrame> {
        self.process_inner(event, None)
    }

    /// Like [`Engine::process`], additionally collecting a
    /// [`CandidateReport`] for every tested point combination - matched or
    /// rejected - into [`EngineFrame::candidates`].
    ///
    /// Used by the recording pipeline (`--record`); allocation happens only
    /// on this path.
    pub fn process_with_diagnostics(&mut self, event: &RawEvent) -> Option<EngineFrame> {
        self.process_inner(event, Some(&mut Vec::new()))
    }

    fn process_inner(
        &mut self,
        event: &RawEvent,
        mut diagnostics: Option<&mut Vec<CandidateReport>>,
    ) -> Option<EngineFrame> {
        self.touch_state.process(event);
        if !event.is_syn_report() {
            return None;
        }
        let mut touches = self.touch_state.active_touches();
        touches.sort_by_key(|touch| touch.slot);
        let matches = match diagnostics.as_mut() {
            Some(out) => {
                matching::find_matches_with_diagnostics(&touches, &self.patterns, &mut **out)
            }
            None => matching::find_matches(&touches, &self.patterns),
        };
        let mut events = self.tracker.update(&matches);
        if touches != self.prev_touches {
            self.prev_touches.clone_from(&touches);
            events.push(EngineEvent::TouchPointsUpdated);
        }
        debug!(
            matches = matches.len(),
            tracked = self.tracker.active().len(),
            "frame processed"
        );
        Some(EngineFrame {
            matches,
            touches,
            events,
            candidates: diagnostics.map(std::mem::take).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use crate::EngineEvent;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::EventType;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::SYN_REPORT;
    use tangible_objects_model::TangiblePattern;
    use tangible_objects_model::Vec2;

    fn triangle_pattern() -> TangiblePattern {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        TangiblePattern::from_points(1, "triangle", &reference, 5.0)
    }

    fn contact(engine: &mut Engine, slot: i32, id: i32, x: i32, y: i32) {
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_SLOT, slot));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, id));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, x));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, y));
    }

    #[test]
    fn frame_is_emitted_on_syn_report() {
        let mut engine = Engine::new(vec![]);
        assert!(
            engine
                .process(&RawEvent::new(EventType::Abs, ABS_MT_SLOT, 0))
                .is_none()
        );
        assert!(
            engine
                .process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
                .is_some()
        );
    }

    #[test]
    fn triangle_is_detected_and_tracked() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);
        let frame = engine.process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0));

        let frame = frame.expect("frame expected on SYN_REPORT");
        assert_eq!(frame.matches.len(), 1);
        assert_eq!(frame.matches[0].pattern_name, "triangle");
        assert!(
            frame
                .events
                .iter()
                .any(|e| matches!(e, EngineEvent::TangibleAdded(_)))
        );
        assert!(
            frame
                .events
                .iter()
                .any(|e| matches!(e, EngineEvent::TouchPointsUpdated))
        );
        assert_eq!(engine.tracked().len(), 1);
    }

    #[test]
    fn process_with_diagnostics_collects_candidates() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);

        let frame = engine
            .process_with_diagnostics(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
            .expect("frame expected on SYN_REPORT");

        assert_eq!(frame.candidates.len(), 1);
        assert!(frame.candidates[0].matched);
        assert_eq!(frame.candidates[0].deviations.len(), 3);
    }

    #[test]
    fn process_leaves_candidates_empty() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);

        let frame = engine
            .process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
            .expect("frame expected on SYN_REPORT");

        assert!(frame.candidates.is_empty());
    }
}
