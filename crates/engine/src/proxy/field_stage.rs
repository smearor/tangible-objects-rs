use std::collections::HashSet;

use tangible_objects_model::FieldSet;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;

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
    suppressed: HashSet<TouchPointSlot>,
}

impl FieldStage {
    /// Creates a field stage with no sets - the daemon syncs them.
    pub fn new() -> Self {
        Self {
            sets: Vec::new(),
            suppressed: HashSet::new(),
        }
    }
}

impl Default for FieldStage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for FieldStage {
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
        self.suppressed = frame
            .touches
            .iter()
            .filter(|touch| {
                self.sets.iter().any(|set| {
                    set.suppress
                        && set
                            .fields
                            .iter()
                            .any(|field| set.field_contains(field, touch))
                })
            })
            .map(|touch| touch.slot)
            .collect();
    }

    fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }
}
