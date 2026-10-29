use super::PatternId;
use super::RefIndices;
use super::RefPoints;
use crate::ClaimedIndices;
use crate::CombinationBuf;
use crate::FrameCount;
use crate::Millimeters;
use crate::PatternName;
use crate::PointSet;
use crate::RotationRad;
use crate::SlotSet;
use crate::TangiblePattern;
use crate::TouchPointSlot;
use crate::TouchSet;
use crate::Vec2;
use std::collections::HashMap;

/// A successful pattern match within a single input frame.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// ID of the matched pattern (assigned at profile load time).
    pub pattern_id: PatternId,
    /// Name of the matched figure profile.
    pub pattern_name: PatternName,
    /// Centroid of the matched touch points.
    pub center: Vec2<Millimeters>,
    /// Estimated figure orientation in radians - correspondence-fitted
    /// when [`Self::correspondence`] is solved, else the
    /// [`PointSet::rotation_of`] estimate.
    pub rotation_rad: RotationRad,
    /// Indices into the frame's touch point list that belong to this figure.
    pub touch_indices: CombinationBuf,
    /// Solved pin identity: `correspondence[i]` is the reference pin of
    /// `touch_indices[i]` (ROTATION.md). Empty when unsolved - no
    /// `ref_points` on the pattern, degenerate geometry, or the
    /// incumbent path that never solves.
    pub correspondence: RefIndices,
    /// The pattern's reference pin geometry, resolved once together
    /// with [`Self::correspondence`] - the tracker persists it so the
    /// incumbent path needs no pattern lookup.
    pub ref_points: Option<RefPoints>,
    /// Whether the match used a partial (`num_points - k`) fingerprint.
    /// `true` also for completed partials - see `claimed_indices`.
    pub partial: bool,
    /// [`Claim`]s of missing-pin completion - measured touches bound
    /// to the reference pins they fill, predicted from the matched
    /// geometry and assigned to this figure even though they did not
    /// satisfy the fingerprint (TRACKING_STABILIZATION.md).
    /// Empty when completion did not run or found nothing.
    pub claimed_indices: ClaimedIndices,
    /// Frames a partial *spawn* must hold before the tangible is
    /// confirmed - carried from the pattern
    /// (SPAWN_AS_PARTIAL.md); ignored for non-partial matches.
    pub spawn_stability: FrameCount,
}

impl PatternMatch {
    /// Whether this match is a partial (`num_points - k`) match.
    ///
    /// Callers that must distinguish certainty levels (e.g. the filter
    /// proxy, which suppresses slots only on exact matches) use this flag
    /// instead of re-inspecting touch counts.
    pub fn is_partial(&self) -> bool {
        self.partial
    }

    /// All slots of this match: matched pins plus claimed pins.
    pub fn all_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.touch_indices
            .iter()
            .copied()
            .chain(self.claimed_indices.touch_indices())
    }

    /// Solves the pin correspondence of this match's measured points
    /// against `pattern`'s reference geometry (ROTATION.md) and
    /// replaces `rotation_rad` with the fitted angle. `kept_refs` are
    /// the reference indices `points` correspond to - the full set for
    /// exact matches, the surviving subset for partials. No-op without
    /// `ref_points` or on degenerate input; the `rotation_of`
    /// estimate then stands.
    pub fn resolve_correspondence(
        &mut self,
        pattern: &TangiblePattern,
        points: PointSet<'_>,
        kept_refs: &RefIndices,
    ) {
        let Some(ref_points) = &pattern.ref_points else {
            return;
        };
        let Some(fit) = ref_points.fit_rotation(kept_refs, points, pattern.tolerance) else {
            return;
        };
        self.rotation_rad = fit.angle;
        self.correspondence = fit.correspondence;
        self.ref_points = Some(ref_points.clone());
    }

    /// The genuine slot→pin binding of this match - `touch_indices`
    /// paired with their solved [`Self::correspondence`] pins and
    /// resolved through `touches` to slot identities (ROTATION.md).
    /// The seed of a persisted `PinMap`.
    pub fn genuine_map(&self, touches: TouchSet<'_>) -> HashMap<TouchPointSlot, crate::RefIndex> {
        self.touch_indices
            .corresponding(&self.correspondence)
            .map(|(touch_index, ref_index)| (touches[touch_index].slot, ref_index))
            .collect()
    }

    /// Resolves `touch_indices` to the slots of the touched points -
    /// the genuine-match set used for incumbent re-matching (claimed
    /// indices excluded: a claimed pin did not satisfy the fingerprint).
    pub fn matched_slots(&self, touches: TouchSet<'_>) -> SlotSet {
        self.touch_indices
            .iter()
            .map(|&index| touches[index].slot)
            .collect()
    }

    /// Resolves `touch_indices` (plus `claimed_indices` when
    /// `bind_claimed`) to the slots of the touched points - the slot
    /// binding a tracked figure holds over its contacts.
    pub fn slots_of(&self, touches: TouchSet<'_>, bind_claimed: bool) -> SlotSet {
        let claimed: Vec<usize> = if bind_claimed {
            self.claimed_indices.touch_indices().collect()
        } else {
            Vec::new()
        };
        self.touch_indices
            .iter()
            .copied()
            .chain(claimed)
            .map(|index| touches[index].slot)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::PatternMatch;
    use crate::Claim;
    use crate::ClaimedIndices;
    use crate::DEFAULT_SPAWN_STABILITY;
    use crate::PatternId;
    use crate::RefIndex;
    use crate::RefIndices;
    use crate::RotationRad;
    use crate::Vec2;

    #[test]
    fn partial_flag_and_all_indices() {
        let mut pattern_match = PatternMatch {
            pattern_id: PatternId::new(1),
            pattern_name: "Figur8".into(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: vec![0, 1, 2].into(),
            correspondence: RefIndices::default(),
            ref_points: None,
            partial: false,
            claimed_indices: ClaimedIndices::default(),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
        };
        assert!(!pattern_match.is_partial());
        assert_eq!(pattern_match.all_indices().count(), 3);
        pattern_match.partial = true;
        pattern_match.claimed_indices = vec![Claim::new(5, RefIndex::new(0))].into();
        assert!(pattern_match.is_partial());
        assert_eq!(
            pattern_match.all_indices().collect::<Vec<usize>>(),
            vec![0, 1, 2, 5]
        );
    }
}
