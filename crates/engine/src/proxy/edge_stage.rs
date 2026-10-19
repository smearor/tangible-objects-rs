use std::collections::HashSet;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;

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
    suppressed: HashSet<TouchPointSlot>,
}

impl EdgeStage {
    /// Creates an edge stage suppressing contacts within `margin` of the
    /// `surface` bounds.
    pub(crate) fn new(margin: Millimeters, surface: Region) -> Self {
        Self {
            margin,
            surface,
            suppressed: HashSet::new(),
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
        self.suppressed = frame
            .touches
            .iter()
            .filter(|touch| !inner.contains(&touch.pos))
            .map(|touch| touch.slot)
            .collect();
    }

    fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }
}
