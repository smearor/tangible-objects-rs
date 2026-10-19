use std::collections::HashSet;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Region;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;

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
    suppressed: HashSet<TouchPointSlot>,
}

impl ZoneStage {
    /// Creates a zone stage suppressing contacts inside `zones`.
    pub(crate) fn new(zones: Vec<Region>) -> Self {
        Self {
            zones,
            suppressed: HashSet::new(),
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
        self.suppressed = frame
            .touches
            .iter()
            .filter(|touch| self.zones.iter().any(|zone| zone.contains(&touch.pos)))
            .map(|touch| touch.slot)
            .collect();
    }

    fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }
}
