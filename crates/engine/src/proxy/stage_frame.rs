use std::collections::HashMap;
use std::collections::HashSet;

use tangible_objects_model::StageName;
use tangible_objects_model::TouchPointSlot;
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
    pub matched: &'a HashSet<TouchPointSlot>,
    /// Active touches this frame.
    pub touches: TouchSet<'a>,
}

/// Matched slots grouped by stage scope (EVENT_PIPELINE.md phase 4) -
/// the `None` key is the general stage for untagged patterns.
pub(crate) type ScopedMatches = HashMap<Option<StageName>, HashSet<TouchPointSlot>>;
