use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;
use tangible_objects_model::SlotSet;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;
use super::stage_metrics::StageMetrics;

/// Suppresses contacts inside the edge margin - ghost contacts some
/// panels emit at the axis bounds (EVENT_PIPELINE.md).
///
/// Empirically (SiS panel): placing a tangible in the middle of the
/// surface makes the controller fabricate extra phantom contacts at the
/// axis bounds - unmatched, so only position can catch them. They pollute
/// matching *and* leak to the OS as ghost taps. A real finger inside the
/// margin dies the same way - positional filtering cannot distinguish -
/// which is why the margin is opt-in and should be kept small.
///
/// Stateless per frame: `classify` recomputes the suppressed set from the
/// frame's touches - an edge contact that moves inward is unsuppressed
/// the same frame. Always-on: edge suppression is device hygiene, not a
/// `FilterMode` policy, so `set_mode` does not reach it and `Off` does
/// not release edge contacts.
///
/// The engine's own `is_edge_ghost` stays in place - ghosts must not
/// pollute matching either; this stage owns the emitted-stream side
/// (until now edge ghosts leaked to the virtual device).
#[derive(Debug)]
pub(crate) struct EdgeStage {
    /// Inset from the surface bounds that counts as the edge band.
    margin: Millimeters,
    /// Usable surface in millimeters (from the device calibration).
    surface: Region,
    /// Slots suppressed this frame - rebuilt by `classify`.
    suppressed: SlotSet,
    /// Per-stage counters - updated in `classify`.
    metrics: StageMetrics,
}

impl EdgeStage {
    /// Creates an edge stage suppressing contacts within `margin` of the
    /// `surface` bounds.
    pub(crate) fn new(margin: Millimeters, surface: Region) -> Self {
        Self {
            margin,
            surface,
            suppressed: SlotSet::default(),
            metrics: StageMetrics::default(),
        }
    }
}

impl Stage for EdgeStage {
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
        let inner = self.surface.inset(self.margin);
        let suppressed = frame.slots_where(|touch| !inner.contains(&touch.pos));
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
    use super::EdgeStage;
    use super::Stage;
    use crate::proxy::stage_frame::StageFrame;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Region;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn surface() -> Region {
        Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(100.0),
            y_max: Millimeters::new(100.0),
        }
    }

    fn touch(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(slot as i32),
            pos: Vec2::new(x, y),
        }
    }

    /// Contacts in the margin are ghost-suppressed; interior contacts
    /// pass through.
    #[test]
    fn classify_suppresses_edge_band_only() {
        let mut stage = EdgeStage::new(Millimeters::new(10.0), surface());
        let matched = SlotSet::default();
        let touches = vec![touch(0, 5.0, 50.0), touch(1, 50.0, 50.0)];
        stage.classify(&StageFrame {
            matched: &matched,
            touches: (&touches).into(),
        });
        assert!(stage.suppresses(TouchPointSlot::new(0)));
        assert!(!stage.suppresses(TouchPointSlot::new(1)));
    }

    /// The stage is stateless - a contact that moved into the interior
    /// is unsuppressed the same frame.
    #[test]
    fn classify_unsuppresses_on_move_inward() {
        let mut stage = EdgeStage::new(Millimeters::new(10.0), surface());
        let matched = SlotSet::default();
        let edge = vec![touch(0, 5.0, 50.0)];
        stage.classify(&StageFrame {
            matched: &matched,
            touches: (&edge).into(),
        });
        let inner = vec![touch(0, 50.0, 50.0)];
        stage.classify(&StageFrame {
            matched: &matched,
            touches: (&inner).into(),
        });
        assert!(!stage.suppresses(TouchPointSlot::new(0)));
    }
}
