use std::collections::HashMap;

use tangible_objects_model::SlotSet;
use tangible_objects_model::StageName;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TouchSet;

/// Per-frame input a stage classifies - the borrowed view the pipeline
/// hands every stage at `SYN_REPORT`.
///
/// `matched` carries the slots the upstream classification marked this
/// frame (exact matches plus field-suppressed contacts); `touches` are
/// the active contact positions in millimeters - positional stages
/// (edge, zone) classify purely on them.
pub(crate) struct StageFrame<'a> {
    /// Slots marked by upstream classification this frame.
    pub matched: &'a SlotSet,
    /// Active touches this frame.
    pub touches: TouchSet<'a>,
}

/// Matched slots grouped by stage scope (EVENT_PIPELINE.md phase 4) -
/// the `None` key is the general stage for untagged patterns.
pub(crate) type ScopedMatches = HashMap<Option<StageName>, SlotSet>;

impl StageFrame<'_> {
    /// Slots of this frame's contacts satisfying `predicate` -
    /// convenience delegate to [`TouchSet::slots_where`] so positional
    /// stages read "the frame's slots where".
    pub fn slots_where(&self, predicate: impl FnMut(&TouchPoint) -> bool) -> SlotSet {
        self.touches.slots_where(predicate)
    }
}
