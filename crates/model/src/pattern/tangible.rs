use super::Distances;
use super::PartialMatch;
use super::PartialReference;
use super::PatternId;
use super::PatternTolerance;
use super::PatternTolerances;
use super::PinCount;
use super::RefIndex;
use super::RefIndices;
use super::RefPoints;
use super::RigidTransform;
use crate::AffinityFactor;
use crate::Claim;
use crate::ClaimPool;
use crate::ClaimedIndices;
use crate::Combination;
use crate::CompletionConfig;
use crate::DEFAULT_SPAWN_STABILITY;
use crate::Deviations;
use crate::FrameCount;
use crate::MatchError;
use crate::MatchScore;
use crate::Millimeters;
use crate::PatternMatch;
use crate::PatternName;
use crate::PointSet;
use crate::Region;
use crate::StageName;
use crate::TouchSet;
use crate::Vec2;
use std::collections::HashSet;

/// A tangible figure pattern defined by invariant pairwise distances.
///
/// For n points the fingerprint consists of all `n*(n-1)/2` pairwise
/// distances, sorted ascending. Matching compares a measured distance set
/// against the fingerprint within a tolerance, which makes the fingerprint
/// position- and rotation-invariant.
#[derive(Debug, Clone)]
pub struct TangiblePattern {
    /// Runtime ID assigned when the profile directory is loaded.
    pub id: PatternId,
    /// Human-readable figure name (from the profile file).
    pub name: PatternName,
    /// Whether a partial match may spawn this tangible
    /// (SPAWN_AS_PARTIAL.md); the incumbent path ignores the flag.
    pub spawn_on_partial: bool,
    /// Frames a partial spawn must hold before the tangible leaves
    /// pending state.
    pub spawn_stability: FrameCount,
    /// Number of contact pins the figure must have.
    pub num_points: PinCount,
    /// Sorted pairwise distances of the reference point set (the fingerprint).
    pub sorted_distances: Distances,
    /// Maximum allowed deviation per distance during matching.
    pub tolerance: PatternTolerance,
    /// Optional per-distance tolerances, parallel to `sorted_distances`.
    /// When empty (or length-mismatched), the scalar `tolerance` applies to
    /// every index - the recorded `p99` suggestions from the analysis crate
    /// are written here.
    pub tolerances: PatternTolerances,
    /// Original reference point positions, if stored in the profile.
    /// Required for partial matching when one contact pin is missing.
    pub ref_points: Option<RefPoints>,
    /// Detection regions restricting where this figure may match
    /// (PROFILE_REGIONS.md). Empty = anywhere on the surface.
    pub regions: Vec<Region>,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md). `None` = the general stage.
    pub stage: Option<StageName>,
}

impl TangiblePattern {
    /// Creates a pattern from known reference point positions.
    pub fn from_points(
        id: impl Into<PatternId>,
        name: &str,
        points: PointSet<'_>,
        tolerance: impl Into<PatternTolerance>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            spawn_on_partial: false,
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            num_points: PinCount::new(points.len()),
            sorted_distances: points.distances(),
            tolerance: tolerance.into(),
            tolerances: PatternTolerances::default(),
            ref_points: Some(points.to_vec().into()),
            regions: Vec::new(),
            stage: None,
        }
    }

    /// Whether `points` may match spatially: every point must lie inside
    /// the union of this pattern's [`Region`]s. With no regions configured
    /// every position is accepted.
    pub fn accepts(&self, points: PointSet<'_>) -> bool {
        self.regions.is_empty()
            || points
                .iter()
                .all(|point| self.regions.iter().any(|region| region.contains(point)))
    }

    /// The effective tolerance for distance index `i`: the per-index value
    /// when `tolerances` is configured, else the scalar `tolerance`.
    pub fn tolerance_at(&self, index: usize) -> PatternTolerance {
        self.tolerances
            .get(index)
            .copied()
            .unwrap_or(self.tolerance)
    }

    /// Checks whether a set of measured points matches this pattern exactly.
    ///
    /// Each deviation is compared against its per-index tolerance
    /// ([`Self::tolerance_at`]); the scalar `tolerance` is the fallback when
    /// no `tolerances` vector is configured.
    pub fn matches(&self, points: PointSet<'_>) -> bool {
        if points.len() != self.num_points {
            return false;
        }
        let measured = points.distances();
        measured.len() == self.sorted_distances.len()
            && measured
                .iter()
                .zip(self.sorted_distances.iter())
                .enumerate()
                .all(|(i, (m, t))| (*m - *t).abs() <= self.tolerance_at(i).as_millimeters())
    }

    /// Normalized deviation score of a measured fingerprint: the largest
    /// deviation-to-tolerance ratio across all distance indices.
    ///
    /// `score <= 1.0` is equivalent to [`Self::matches`] on the same point
    /// set - but the score additionally ranks candidates (lower is closer)
    /// and quantifies near-misses, which the score-based matcher and the
    /// offline analysis use.
    pub fn match_score(&self, points: PointSet<'_>) -> MatchScore {
        self.score_of(&points.distances().deviations(&self.sorted_distances))
    }

    /// Builds the [`PatternMatch`] for a matched point combination:
    /// centroid center, `rotation_of` estimate (replaced later for
    /// exact winners via [`PatternMatch::resolve_correspondence`]),
    /// and the `(partial)` name marker on reduced-fingerprint matches.
    pub fn build_match(
        &self,
        points: PointSet<'_>,
        combo: Combination<'_>,
        partial: bool,
        claimed: ClaimedIndices,
    ) -> PatternMatch {
        let center = Vec2::centroid(points.as_slice());
        let name = if partial {
            format!("{} (partial)", self.name).into()
        } else {
            self.name.clone()
        };
        PatternMatch {
            pattern_id: self.id,
            pattern_name: name,
            spawn_stability: self.spawn_stability,
            center,
            rotation_rad: points.rotation_of(&center),
            touch_indices: combo.into(),
            correspondence: RefIndices::default(),
            ref_points: None,
            partial,
            claimed_indices: claimed,
        }
    }

    /// Incumbent re-match on bound slots (TRACKING_STABILIZATION.md):
    /// `combo` is verified against the affinity-scaled score
    /// (`match_score <= factor`) - the slot binding already proves
    /// identity, so motion distortion must not lose the match.
    pub fn try_exact(
        &self,
        combo: Combination<'_>,
        touches: TouchSet<'_>,
        completion: &CompletionConfig,
    ) -> Option<PatternMatch> {
        let points = touches.positions_of(combo.as_slice());
        (self.match_score((&points).into()).as_f64() <= completion.affinity_factor.as_f64()
            && self.accepts((&points).into()))
        .then(|| self.build_match(points.as_point_set(), combo, false, Default::default()))
    }

    /// Like [`Self::try_exact`] for a partial (`num_points - 1`)
    /// combination; also claims the predicted missing pin(s) from
    /// `pool` when completion is enabled.
    pub fn try_partial(
        &self,
        combo: Combination<'_>,
        touches: TouchSet<'_>,
        completion: &CompletionConfig,
        pool: ClaimPool<'_>,
    ) -> Option<PatternMatch> {
        let points = touches.positions_of(combo.as_slice());
        let missing = self.num_points.as_usize().checked_sub(combo.len())?;
        self.matches_partial_missing_scaled((&points).into(), missing, completion.affinity_factor)
            .filter(|_| self.accepts((&points).into()))
            .map(|matched| {
                let claimed = self.claim_missing(
                    &matched.kept_indices,
                    points.as_point_set(),
                    combo,
                    pool,
                    touches,
                    completion,
                );
                self.build_match(points.as_point_set(), combo, true, claimed)
            })
    }

    /// Claims the predicted positions of a partial match's missing
    /// pins (TRACKING_STABILIZATION.md).
    ///
    /// `kept_refs` are the reference indices that matched;
    /// `points`/`combo` the measured points. For every missing
    /// reference pin the expected position is predicted via
    /// [`Self::predict_missing`] and the nearest free touch inside
    /// `radius_factor` × tolerance is claimed.
    pub fn claim_missing(
        &self,
        kept_refs: &RefIndices,
        points: PointSet<'_>,
        combo: Combination<'_>,
        pool: ClaimPool<'_>,
        touches: TouchSet<'_>,
        completion: &CompletionConfig,
    ) -> ClaimedIndices {
        let radius = self.tolerance.as_f64() * completion.radius_factor;
        let mut claimed: Vec<Claim> = Vec::new();
        let mut taken: HashSet<usize> = combo.iter().copied().collect();
        for missing in kept_refs.complement(self.num_points) {
            let Some(predicted) = self.predict_missing(kept_refs, points, missing) else {
                continue;
            };
            let nearest = pool
                .iter()
                .filter(|index| !taken.contains(index))
                .min_by(|&&a, &&b| {
                    touches[a]
                        .pos
                        .distance(&predicted)
                        .total_cmp(&touches[b].pos.distance(&predicted))
                });
            if let Some(&index) =
                nearest.filter(|&&index| touches[index].pos.distance(&predicted).as_f64() <= radius)
            {
                claimed.push(Claim::new(index, missing));
                taken.insert(index);
            }
        }
        claimed.into()
    }

    /// Score of an already-computed deviation vector - `max_i dev[i] /
    /// tolerance_at(i)`; 0.0 for empty or length-mismatched input.
    pub fn score_of(&self, deviations: &Deviations) -> MatchScore {
        deviations.max_score(|i| self.tolerance_at(i).as_millimeters())
    }

    /// Total absolute deviation across all distances (lower is better).
    pub fn match_error(&self, points: PointSet<'_>) -> MatchError {
        points
            .distances()
            .deviations(&self.sorted_distances)
            .total()
    }

    /// Absolute per-index deviation between a measured and a reference
    /// fingerprint; empty when the lengths differ.
    pub fn deviations(measured: &Distances, reference: &Distances) -> Deviations {
        measured.deviations(reference)
    }

    /// Widened acceptance tolerance for partial (missing-pin) matches -
    /// a reduced fingerprint needs more slack than an exact one
    /// (SPAWN_AS_PARTIAL.md): `self.tolerance` unscaled.
    pub fn partial_tolerance(&self) -> Millimeters {
        self.tolerance.partial(AffinityFactor::neutral())
    }

    /// Fingerprint of the reference point set with pin `skip` removed.
    ///
    /// Returns the kept reference indices and their sorted pairwise
    /// distances; `None` when no reference points are stored, their count
    /// differs from `num_points` or `skip` is out of range.
    pub fn partial_reference(&self, skip: RefIndex) -> Option<PartialReference> {
        self.partial_reference_multi(&vec![skip].into())
    }

    /// Like [`Self::partial_reference`] with `skips` pins removed -
    /// needed for `num_points - 2` completion candidates.
    pub fn partial_reference_multi(&self, skips: &RefIndices) -> Option<PartialReference> {
        let ref_points = self.ref_points.as_ref()?;
        if ref_points.len() != self.num_points
            || skips.iter().any(|skip| skip.as_usize() >= self.num_points)
            || skips.is_empty()
        {
            return None;
        }
        let subset: Vec<Vec2<Millimeters>> = ref_points
            .iter()
            .enumerate()
            .filter(|(index, _)| !skips.contains(&RefIndex::new(*index)))
            .map(|(_, point)| *point)
            .collect();
        let indices = skips.complement(self.num_points);
        Some(PartialReference {
            kept_indices: indices,
            distances: PointSet::new(&subset).distances(),
        })
    }

    /// Partial match: tests whether `points` (`num_points - 1` contacts) fit a
    /// subset of the reference points with one pin missing (e.g. a tilting
    /// object). Every possibility of dropping one reference point is tried.
    /// The scalar tolerance is raised by 50% to account for the higher
    /// uncertainty - per-index `tolerances` cannot apply here because
    /// dropping a pin re-indexes the distance fingerprint.
    ///
    /// Returns the matched reference indices and the total error.
    pub fn matches_partial(&self, points: PointSet<'_>) -> Option<PartialMatch> {
        self.matches_partial_missing(points, 1)
    }

    /// Generalized partial match with `missing` dropped pins (1 or 2).
    /// The returned indices are the *kept* reference points, ordered like
    /// the reference - the complement are the missing pins, used by
    /// missing-pin completion to predict their expected positions.
    pub fn matches_partial_missing(
        &self,
        points: PointSet<'_>,
        missing: usize,
    ) -> Option<PartialMatch> {
        let ref_pts = self.ref_points.as_ref()?;
        if self.num_points == 0
            || missing == 0
            || missing >= self.num_points
            || points.len() != self.num_points - missing
            || ref_pts.len() != self.num_points
        {
            return None;
        }
        self.matches_partial_missing_scaled(points, missing, AffinityFactor::neutral())
    }

    /// Like [`Self::matches_partial_missing`], with the partial tolerance
    /// scaled by `tol_scale` - incumbent re-matching relaxes the threshold
    /// (slot binding already proves identity); `1.0` keeps the default.
    pub fn matches_partial_missing_scaled(
        &self,
        points: PointSet<'_>,
        missing: usize,
        tol_scale: AffinityFactor,
    ) -> Option<PartialMatch> {
        self.ref_points.as_ref()?;
        let partial_tolerance = self.tolerance.partial(tol_scale);
        let measured = points.distances();
        let mut best: Option<PartialMatch> = None;

        for skips in self.num_points.skip_combinations(missing) {
            let Some(reference) = self.partial_reference_multi(&skips.ref_indices()) else {
                continue;
            };

            let deviations = Self::deviations(&measured, &reference.distances);
            if deviations.len() == reference.distances.len()
                && deviations
                    .iter()
                    .all(|d| d.as_f64() <= partial_tolerance.as_f64())
            {
                let candidate = PartialMatch {
                    kept_indices: reference.kept_indices,
                    match_error: deviations.total(),
                };
                if best
                    .as_ref()
                    .is_none_or(|b| candidate.match_error < b.match_error)
                {
                    best = Some(candidate);
                }
            }
        }
        best
    }

    /// Predicts the screen position of a missing reference pin from a
    /// partial match's geometry.
    ///
    /// `kept_refs` are the reference indices that matched (from
    /// [`Self::matches_partial_missing`]); `measured` are the frame
    /// positions of the kept pins in the same order. The missing pin's
    /// expected position is estimated by fitting a rigid transform
    /// (rotation + translation, no scale) ref→measured.
    ///
    /// Returns `None` when the correspondence between kept reference and
    /// measured points is ambiguous within `tolerance`.
    pub fn predict_missing(
        &self,
        kept_refs: &RefIndices,
        measured: PointSet<'_>,
        missing: RefIndex,
    ) -> Option<Vec2<Millimeters>> {
        let ref_points = self.ref_points.as_ref()?;
        if missing.as_usize() >= self.num_points
            || kept_refs.len() != measured.len()
            || kept_refs.len() < 2
            || kept_refs.iter().any(|i| i.as_usize() >= self.num_points)
        {
            return None;
        }
        let kept_ref_points = kept_refs.points_of(ref_points);
        // Try every correspondence consistent with the distance
        // multisets (symmetric figures admit several) and keep the
        // transform with the smallest residual.
        let mut best: Option<(RigidTransform, Millimeters)> = None;
        for mapping in kept_ref_points.correspondences(measured, self.tolerance) {
            let Some(transform) =
                kept_ref_points.similarity_transform(measured, mapping.as_combination())
            else {
                continue;
            };
            let residual = Millimeters::new(
                transform
                    .residual(&kept_ref_points, measured, mapping.as_combination())
                    .sqrt(),
            );
            if best.as_ref().is_none_or(|(_, r)| residual < *r) {
                best = Some((transform, residual));
            }
        }
        best.map(|(transform, _)| transform.apply(ref_points[missing.as_usize()]))
    }
}

#[cfg(test)]
mod tests {
    use super::TangiblePattern;
    use crate::Millimeters;
    use crate::Vec2;

    fn triangle() -> Vec<Vec2<Millimeters>> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ]
    }

    #[test]
    fn fingerprint_contains_all_pairwise_distances() {
        let pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        assert_eq!(pattern.sorted_distances.len(), 3);
    }

    #[test]
    fn same_geometry_different_pose_matches() {
        let pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        // Same triangle, translated and rotated.
        let rotated = vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 150.0),
            Vec2::new(150.0, 100.0),
        ];
        assert!(pattern.matches((&rotated).into()));
    }

    #[test]
    fn wrong_geometry_does_not_match() {
        let pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        let other = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(0.0, 60.0),
        ];
        assert!(!pattern.matches((&other).into()));
    }

    #[test]
    fn wrong_point_count_does_not_match() {
        let pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        assert!(!pattern.matches((&triangle()[..2]).into()));
    }

    #[test]
    fn partial_match_with_missing_pin() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        assert!(pattern.matches_partial((&square[..3]).into()).is_some());
    }

    #[test]
    fn per_index_tolerances_refine_scalar() {
        // Fingerprint [50, 50, ~70.7]; indices 1–2 generous, index 0 strict.
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 1.0);
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()].into();
        // Stretched third contact: deviations [0, ~15, ~11] - the shortest
        // edge is exact, the longer ones sit inside their 20 mm budget.
        // Rejected under scalar 1.0, accepted per-index.
        let stretched = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 65.0),
        ];
        let scalar_only = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 1.0);
        assert!(!scalar_only.matches((&stretched).into()));
        assert!(pattern.matches((&stretched).into()));
    }

    #[test]
    fn per_index_tolerances_still_reject_short_edges() {
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 1.0);
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()].into();
        // The shortest edge shrinks (45 vs 50): index 0 deviates by 5 mm and
        // exceeds its strict 1 mm budget even though indices 1–2 would allow
        // the deformation.
        let warped = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(45.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        assert!(!pattern.matches((&warped).into()));
    }

    #[test]
    fn mismatched_tolerances_length_falls_back_to_scalar() {
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        pattern.tolerances = vec![0.1.into()].into(); // too few - must not apply
        let slightly_off = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(53.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        assert!(pattern.matches((&slightly_off).into())); // scalar 5.0 still applies
    }

    #[test]
    fn match_score_marks_boundary_and_near_misses() {
        let pattern = TangiblePattern::from_points(1, "triangle", (&triangle()).into(), 5.0);
        assert_eq!(pattern.match_score((&triangle()).into()).as_f64(), 0.0);
        // A 2.5 mm deviation on one edge is score 0.5 with scalar 5.0.
        let slightly_off = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(52.5, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let score = pattern.match_score((&slightly_off).into());
        assert!(score.as_f64() > 0.4 && score.as_f64() < 0.6);
    }

    #[test]
    fn partial_match_requires_ref_points() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        pattern.ref_points = None;
        assert!(pattern.matches_partial((&square[..3]).into()).is_none());
    }
}
