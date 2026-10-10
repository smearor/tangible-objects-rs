use tangible_objects_model::Calibration;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::EngineFrame;
use crate::TouchState;
use crate::Tracker;
use crate::matching;

/// Default EMA smoothing factor for tracked poses.
pub const DEFAULT_EMA_ALPHA: f64 = 0.4;

/// Default association radius between a match and a tracked tangible, in mm.
pub const DEFAULT_MAX_CENTER_DISTANCE: f64 = 200.0;

/// Default number of frames a tangible survives without a match.
pub const DEFAULT_MAX_FRAMES_WITHOUT_MATCH: u32 = 30;

/// Headless detection engine: slot state machine + pattern matching +
/// frame-to-frame tracking.
///
/// Feed [`RawEvent`]s via [`Engine::process`]; every `SYN_REPORT` produces an
/// [`EngineFrame`] with the frame's matches and updates the tracker.
pub struct Engine {
    touch_state: TouchState,
    patterns: Vec<TangiblePattern>,
    tracker: Tracker,
    calibration: Option<Calibration>,
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
    /// Returns `Some` with the frame's matches on each `SYN_REPORT`,
    /// `None` for all other events.
    pub fn process(&mut self, event: &RawEvent) -> Option<EngineFrame> {
        self.touch_state.process(event);
        if event.is_syn_report() {
            let touches = self.touch_state.active_touches();
            let matches = matching::find_matches(&touches, &self.patterns);
            self.tracker.update(&matches);
            Some(EngineFrame { matches })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
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

        let matches = frame.map(|f| f.matches).unwrap_or_default();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_name, "triangle");
        assert_eq!(engine.tracked().len(), 1);
    }
}
