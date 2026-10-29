use tangible_objects_model::ContactCount;
use tangible_objects_model::FrameCount;
use tangible_objects_model::SlotSet;

/// Per-stage counters (EVENT_PIPELINE.md phase 6).
///
/// Each stage owns one and updates it inside `classify` - global
/// `SessionMetrics` counters stay the aggregate; `suppressed` is the
/// gauge answering "which stage is eating contacts right now".
#[derive(Debug, Default, Clone)]
pub struct StageMetrics {
    /// Frames this stage classified.
    pub frames: FrameCount,
    /// Slots suppressed in the current frame (gauge, not cumulative).
    pub suppressed: ContactCount,
    /// Cumulative slot-suppressions - a slot suppressed again after a
    /// release counts anew.
    pub suppressed_total: ContactCount,
    /// Cumulative matched slots fed to this stage's `classify`
    /// (scope-filtered input).
    pub matched_total: ContactCount,
}

impl StageMetrics {
    /// Records a rebuilt per-frame suppression set: bumps `frames`,
    /// counts the newly suppressed slots into `suppressed_total`,
    /// refreshes the `suppressed` gauge and swaps `new` into the stage's
    /// set - the shared tail of rebuild-style `classify` impls.
    pub fn record_frame(&mut self, suppressed: &mut SlotSet, new: SlotSet) {
        self.frames += 1;
        self.suppressed_total += new.difference(suppressed).count().into();
        self.suppressed = new.len().into();
        *suppressed = new;
    }
}

#[cfg(test)]
mod tests {
    use super::StageMetrics;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::TouchPointSlot;

    /// `record_frame` bumps the frame counter, refreshes the gauge and
    /// swaps the suppressed set.
    #[test]
    fn record_frame_updates_counters_and_set() {
        let mut metrics = StageMetrics::default();
        let mut suppressed = SlotSet::default();
        metrics.record_frame(&mut suppressed, [TouchPointSlot::new(0)].into());
        assert_eq!(metrics.frames.as_u64(), 1);
        assert_eq!(metrics.suppressed.0, 1);
        assert!(suppressed.contains(&TouchPointSlot::new(0)));
    }

    /// `suppressed_total` counts only newly suppressed slots - a slot
    /// suppressed again the next frame does not re-count.
    #[test]
    fn suppressed_total_counts_only_new_slots() {
        let mut metrics = StageMetrics::default();
        let mut suppressed = SlotSet::default();
        metrics.record_frame(&mut suppressed, [TouchPointSlot::new(0)].into());
        metrics.record_frame(&mut suppressed, [TouchPointSlot::new(0)].into());
        assert_eq!(metrics.suppressed_total.0, 1);
        // A release + re-suppression counts anew.
        metrics.record_frame(&mut suppressed, SlotSet::default());
        metrics.record_frame(&mut suppressed, [TouchPointSlot::new(0)].into());
        assert_eq!(metrics.suppressed_total.0, 2);
    }
}
