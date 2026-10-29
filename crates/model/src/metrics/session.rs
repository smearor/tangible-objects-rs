use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::ContactCount;
use crate::FrameCount;
use crate::PatternMatch;
use crate::TangibleCount;
use crate::TangibleId;
use crate::TouchSet;

use crate::TouchPointSlot;
use crate::TrackedSet;
use crate::TrackingId;

use super::PatternMetrics;

/// Accumulated recognition metrics of a session.
///
/// Folded over every published frame; readable through the snapshot and
/// all interfaces (`metrics`/`reset_metrics`). The counters replace the
/// ad-hoc analysis previously run on `.jsonl` recordings - uptime,
/// re-ID churn, match quality and per-slot contact stability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total frames processed.
    pub frames: FrameCount,
    /// Frames with at least one tracked tangible (uptime numerator).
    pub tracked_frames: FrameCount,
    /// Total touch contacts observed.
    pub contacts: ContactCount,
    /// Frames where the touch set changed (activity measure).
    pub active_frames: FrameCount,
    /// Frames that ran the generic matching pass
    /// (SKIP_UNCHANGED.md): `frames - generic_frames` is the
    /// idle-skip count - the gate's observability.
    pub generic_frames: FrameCount,
    /// Distinct tracking IDs observed per slot - counts pin flapping
    /// (`1` = perfectly stable contact).
    pub slot_churn: HashMap<TouchPointSlot, ContactCount>,
    /// Per-pattern counters keyed by pattern name.
    pub patterns: HashMap<crate::PatternName, PatternMetrics>,
    /// Distinct tangible identities ever tracked (churn numerator).
    pub tangible_count: TangibleCount,

    /// Internal: slots' tracking-ID sets (source of `slot_churn`).
    #[serde(skip)]
    slot_ids: HashMap<TouchPointSlot, HashSet<TrackingId>>,
    /// Internal: distinct IDs seen (source of `tangible_count`).
    #[serde(skip)]
    seen_ids: HashSet<TangibleId>,
    /// Internal: distinct IDs per pattern name (source of
    /// `PatternMetrics::tangible_count`).
    #[serde(skip)]
    pattern_ids: HashMap<crate::PatternName, HashSet<TangibleId>>,
}

impl SessionMetrics {
    /// Folds one published frame into the counters.
    ///
    /// `touches_changed` is `true` when the frame produced a
    /// `TouchPointsUpdated` event (the caller knows; keeps this type
    /// free of engine-event dependencies). `ran_generic` is the frame's
    /// generic-pass flag (`EngineFrame::ran_generic`).
    pub fn record(
        &mut self,
        touches: TouchSet<'_>,
        matches: &[PatternMatch],
        tracked: TrackedSet<'_>,
        touches_changed: bool,
        ran_generic: bool,
    ) {
        self.frames += 1;
        if !tracked.is_empty() {
            self.tracked_frames += 1;
        }
        if touches_changed {
            self.active_frames += 1;
        }
        if ran_generic {
            self.generic_frames += 1;
        }
        self.contacts += ContactCount::from(touches.len());

        for touch in touches {
            self.slot_ids
                .entry(touch.slot)
                .or_default()
                .insert(touch.tracking_id);
        }
        self.slot_churn = self
            .slot_ids
            .iter()
            .map(|(slot, ids)| (*slot, ContactCount::from(ids.len())))
            .collect();

        for tangible in tracked {
            self.seen_ids.insert(tangible.id);
            let entry = self
                .patterns
                .entry(tangible.pattern_name.clone())
                .or_default();
            entry.tracked_frames += 1;
            let new_id = self
                .pattern_ids
                .entry(tangible.pattern_name.clone())
                .or_default()
                .insert(tangible.id);
            if new_id {
                entry.tangible_count =
                    TangibleCount::new(self.pattern_ids[&tangible.pattern_name].len() as u64);
            }
        }
        self.tangible_count = TangibleCount::new(self.seen_ids.len() as u64);

        for pattern_match in matches {
            let entry = self
                .patterns
                .entry(pattern_match.pattern_name.clone())
                .or_default();
            if pattern_match.is_partial() {
                entry.partial_match_frames += 1;
            } else {
                entry.exact_match_frames += 1;
            }
            entry.claimed_pins += ContactCount::from(pattern_match.claimed_indices.len());
        }
    }

    /// Resets all counters (e.g. at the start of a new test series).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::SessionMetrics;
    use crate::DEFAULT_SPAWN_STABILITY;
    use crate::FrameCount;
    use crate::PatternId;
    use crate::PatternMatch;
    use crate::RefIndices;
    use crate::RotationRad;
    use crate::TangibleId;
    use crate::TouchPoint;
    use crate::TouchPointSlot;
    use crate::TrackedTangible;
    use crate::TrackingId;
    use crate::Vec2;

    fn touch(slot: u32, id: i32) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(id),
            pos: Vec2::new(0.0, 0.0),
        }
    }

    fn matcher(partial: bool) -> PatternMatch {
        PatternMatch {
            pattern_id: PatternId::new(1),
            pattern_name: "fig".into(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: vec![0].into(),
            correspondence: RefIndices::default(),
            ref_points: None,
            partial,
            claimed_indices: Default::default(),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
        }
    }

    fn tracked(id: u32) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(id),
            pattern_id: PatternId::new(1),
            pattern_name: "fig".into(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            frames_since_match: FrameCount::new(0),
        }
    }

    #[test]
    fn accumulates_uptime_churn_and_matches() {
        let mut metrics = SessionMetrics::default();
        metrics.record(
            (&[touch(0, 1)]).into(),
            &[matcher(false)],
            (&[tracked(0)]).into(),
            true,
            true,
        );
        metrics.record(
            (&[touch(0, 9)]).into(),
            &[matcher(true)],
            (&[tracked(0)]).into(),
            true,
            true,
        );
        metrics.record((&[]).into(), &[], (&[]).into(), false, false);

        assert_eq!(metrics.frames, FrameCount::new(3));
        assert_eq!(metrics.tracked_frames, FrameCount::new(2));
        assert_eq!(metrics.active_frames, FrameCount::new(2));
        assert_eq!(metrics.generic_frames, FrameCount::new(2));
        assert_eq!(metrics.contacts.as_u32(), 2);
        assert_eq!(metrics.tangible_count.as_u64(), 1);
        assert_eq!(metrics.slot_churn[&TouchPointSlot::new(0)].as_u32(), 2);
        let pattern = &metrics.patterns[&"fig".into()];
        assert_eq!(pattern.tracked_frames, FrameCount::new(2));
        assert_eq!(pattern.exact_match_frames, FrameCount::new(1));
        assert_eq!(pattern.partial_match_frames, FrameCount::new(1));

        metrics.reset();
        assert_eq!(metrics.frames, FrameCount::new(0));
        assert!(metrics.patterns.is_empty());
    }
}
