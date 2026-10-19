use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPointSlot;

use super::stage_frame::StageFrame;

/// One stage of the event pipeline (EVENT_PIPELINE.md).
///
/// A stage owns its suppression *policy* - which slots count as
/// suppressed this frame and how the decision evolves over frames -
/// while [`super::EmitBoundary`] owns the mechanism (dropping, retraction,
/// guarded withholding). The boundary merges the per-stage views through
/// this trait; `EmitBoundary::finish` is the only consumer.
pub(crate) trait Stage {
    /// The stage's suppression policy for mechanism merges - stages
    /// without a `FilterMode` concept report `Immediate` (always-on
    /// suppression with no gate).
    fn mode(&self) -> FilterMode;

    /// Consecutive matched frames before suppression - only meaningful
    /// for gated tangible stages; gateless stages report 0.
    fn stability_gate(&self) -> FrameCount;

    /// Whether `slot` is suppressed under this stage's policy right now -
    /// the check the boundary applies per event.
    fn suppresses(&self, slot: TouchPointSlot) -> bool;

    /// Raw suppressed-set membership - the guarded-release decision
    /// already knows the policy.
    fn was_suppressed(&self, slot: TouchPointSlot) -> bool;

    /// Currently suppressed slots - unioned by the boundary for
    /// retraction.
    fn suppressed_slots(&self) -> Box<dyn Iterator<Item = TouchPointSlot> + '_>;

    /// Advances the stage's classification by one frame.
    fn classify(&mut self, frame: &StageFrame<'_>);

    /// Releases `slot` - a lifting contact frees it for a new tangible.
    fn release(&mut self, slot: TouchPointSlot);

    /// `SetFilterMode` propagation - only tangible stages carry a
    /// `FilterMode`; always-on stages ignore the command.
    fn set_mode(&mut self, _mode: FilterMode, _stability: Option<FrameCount>) {}
}
