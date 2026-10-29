use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Region;
use tangible_objects_model::SlotSet;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;
use super::stage_metrics::StageMetrics;

/// Suppresses every contact inside the declared zones - play-area
/// masking (EVENT_PIPELINE.md).
///
/// The inverse of a profile's `regions` constraint: regions bound where
/// a *pattern may match*, a zone stage suppresses *any* contact inside
/// its areas regardless of classification - e.g. a figure parking zone
/// whose pins must never reach the OS even when no profile matches.
///
/// Stateless per frame like [`super::EdgeStage`]: `classify` rebuilds the
/// suppressed set; always-on, `set_mode` does not reach it.
#[derive(Debug)]
pub(crate) struct ZoneStage {
    /// Masked areas in millimeters - any contact inside is suppressed.
    zones: Vec<Region>,
    /// Slots suppressed this frame - rebuilt by `classify`.
    suppressed: SlotSet,
    /// Per-stage counters - updated in `classify`.
    metrics: StageMetrics,
}

impl ZoneStage {
    /// Creates a zone stage suppressing contacts inside `zones`.
    pub(crate) fn new(zones: Vec<Region>) -> Self {
        Self {
            zones,
            suppressed: SlotSet::default(),
            metrics: StageMetrics::default(),
        }
    }
}

impl Stage for ZoneStage {
    fn mode(&self) -> FilterMode {
        FilterMode::Immediate
    }

    fn stability_gate(&self) -> FrameCount {
        FrameCount::new(0)
    }

    fn suppresses(&self, slot: TouchPointSlot) -> bool {
        self.suppressed.contains(&slot)
    }

    fn was_suppressed(&self, slot: TouchPointSlot) -> bool {
        self.suppressed.contains(&slot)
    }

    fn suppressed_slots(&self) -> Box<dyn Iterator<Item = TouchPointSlot> + '_> {
        Box::new(self.suppressed.iter().copied())
    }

    fn classify(&mut self, frame: &StageFrame<'_>) {
        let suppressed =
            frame.slots_where(|touch| self.zones.iter().any(|zone| zone.contains(&touch.pos)));
        self.metrics.record_frame(&mut self.suppressed, suppressed);
    }

    fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }

    fn metrics(&self) -> &StageMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::Stage;
    use super::ZoneStage;
    use crate::proxy::stage_frame::StageFrame;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Region;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn zone() -> Region {
        Region {
            x_min: Millimeters::new(10.0),
            y_min: Millimeters::new(10.0),
            x_max: Millimeters::new(20.0),
            y_max: Millimeters::new(20.0),
        }
    }

    fn touch(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(slot as i32),
            pos: Vec2::new(x, y),
        }
    }

    fn classify(stage: &mut ZoneStage, touches: &[TouchPoint]) {
        let matched = SlotSet::default();
        stage.classify(&StageFrame {
            matched: &matched,
            touches: touches.into(),
        });
    }

    /// A contact inside the zone is suppressed; one outside passes.
    #[test]
    fn classify_suppresses_inside_zone_only() {
        let mut stage = ZoneStage::new(vec![zone()]);
        let touches = vec![touch(0, 15.0, 15.0), touch(1, 50.0, 50.0)];
        classify(&mut stage, &touches);
        assert!(stage.suppresses(TouchPointSlot::new(0)));
        assert!(!stage.suppresses(TouchPointSlot::new(1)));
    }

    /// The stage is stateless - a contact that moved out of the zone
    /// is unsuppressed the same frame.
    #[test]
    fn classify_unsuppresses_on_move_out() {
        let mut stage = ZoneStage::new(vec![zone()]);
        let inside = vec![touch(0, 15.0, 15.0)];
        classify(&mut stage, &inside);
        let outside = vec![touch(0, 50.0, 50.0)];
        classify(&mut stage, &outside);
        assert!(!stage.suppresses(TouchPointSlot::new(0)));
    }

    /// `release` drops the slot from the suppressed set.
    #[test]
    fn release_clears_suppression() {
        let mut stage = ZoneStage::new(vec![zone()]);
        let touches = vec![touch(0, 15.0, 15.0)];
        classify(&mut stage, &touches);
        stage.release(TouchPointSlot::new(0));
        assert!(!stage.suppresses(TouchPointSlot::new(0)));
    }
}
