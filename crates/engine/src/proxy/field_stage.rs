use tangible_objects_model::FieldSet;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::SlotSet;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;
use super::stage_metrics::StageMetrics;

/// Suppresses contacts inside `suppress`-enabled field sets
/// (SINGLE_CONTACT_FIELD.md, EVENT_PIPELINE.md phase 5).
///
/// Owns the *suppression* view only - field occupancy events
/// (`FieldPlaced`/`Lifted`/`CountChanged`) keep flowing through the
/// engine's `FieldTracker` into `EngineFrame.events`; wiring stage
/// annotations into the domain-event stream is phase 6 work. The
/// suppression itself is stateless (point-in-field), so `classify`
/// rebuilds the set each frame.
///
/// Always-on like the positional stages: field suppression is declared
/// by the set's `suppress` flag, not by a `FilterMode`.
#[derive(Debug)]
pub(crate) struct FieldStage {
    /// Registered field sets - mirrored from the engine's registry via
    /// `set_fields`.
    sets: Vec<FieldSet>,
    /// Slots suppressed this frame - rebuilt by `classify`.
    suppressed: SlotSet,
    /// Per-stage counters - updated in `classify`.
    metrics: StageMetrics,
}

impl FieldStage {
    /// Creates a field stage with no sets - the daemon syncs them.
    pub fn new() -> Self {
        Self {
            sets: Vec::new(),
            suppressed: SlotSet::default(),
            metrics: StageMetrics::default(),
        }
    }
}

impl Default for FieldStage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for FieldStage {
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
        let suppressed = frame.slots_where(|touch| {
            self.sets.iter().any(|set| {
                set.suppress
                    && set
                        .fields
                        .iter()
                        .any(|field| set.field_contains(field, touch))
            })
        });
        self.metrics.record_frame(&mut self.suppressed, suppressed);
    }

    fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }

    /// Replaces the stage's field sets - called by the pipeline when the
    /// engine's registry changes (initial `--fields` load and
    /// `RegisterFieldSet`/`UnregisterFieldSet` commands).
    fn set_fields(&mut self, sets: Vec<FieldSet>) {
        self.sets = sets;
    }

    /// Marker - the pipeline creates the field stage lazily via
    /// [`super::Pipeline::set_field_sets`].
    fn is_field_stage(&self) -> bool {
        true
    }

    fn metrics(&self) -> &StageMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::FieldStage;
    use super::Stage;
    use crate::proxy::stage_frame::StageFrame;
    use tangible_objects_model::Field;
    use tangible_objects_model::FieldSet;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Region;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn cell() -> Field {
        Field {
            name: "cell".into(),
            region: Region {
                x_min: Millimeters::new(10.0),
                y_min: Millimeters::new(10.0),
                x_max: Millimeters::new(20.0),
                y_max: Millimeters::new(20.0),
            },
        }
    }

    fn set(suppress: bool) -> FieldSet {
        FieldSet {
            name: "board".into(),
            window: None,
            stability: Default::default(),
            fields: vec![cell()],
            suppress,
        }
    }

    fn touch(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(slot as i32),
            pos: Vec2::new(x, y),
        }
    }

    fn classify(stage: &mut FieldStage, touches: &[TouchPoint]) {
        let matched = SlotSet::default();
        stage.classify(&StageFrame {
            matched: &matched,
            touches: touches.into(),
        });
    }

    /// A `suppress` set masks contacts inside its fields.
    #[test]
    fn classify_suppresses_inside_suppress_set() {
        let mut stage = FieldStage::new();
        stage.set_fields(vec![set(true)]);
        let touches = vec![touch(0, 15.0, 15.0), touch(1, 50.0, 50.0)];
        classify(&mut stage, &touches);
        assert!(stage.suppresses(TouchPointSlot::new(0)));
        assert!(!stage.suppresses(TouchPointSlot::new(1)));
    }

    /// A non-`suppress` set leaves contacts inside its fields
    /// un-suppressed - occupancy events still flow via `FieldTracker`.
    #[test]
    fn classify_ignores_non_suppress_set() {
        let mut stage = FieldStage::new();
        stage.set_fields(vec![set(false)]);
        let touches = vec![touch(0, 15.0, 15.0)];
        classify(&mut stage, &touches);
        assert!(!stage.suppresses(TouchPointSlot::new(0)));
    }

    /// No registered sets suppress nothing.
    #[test]
    fn empty_sets_suppress_nothing() {
        let mut stage = FieldStage::new();
        let touches = vec![touch(0, 15.0, 15.0)];
        classify(&mut stage, &touches);
        assert!(!stage.suppresses(TouchPointSlot::new(0)));
    }
}
