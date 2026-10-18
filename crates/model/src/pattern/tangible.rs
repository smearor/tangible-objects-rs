use super::PartialMatch;
use super::PartialReference;
use super::PatternId;
use super::PatternTolerance;
use super::PinCount;
use crate::FrameCount;
use crate::Millimeters;
use crate::PatternName;
use crate::PointSet;
use crate::Region;
use crate::Vec2;

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
    pub sorted_distances: Vec<Millimeters>,
    /// Maximum allowed deviation per distance during matching.
    pub tolerance: PatternTolerance,
    /// Optional per-distance tolerances, parallel to `sorted_distances`.
    /// When empty (or length-mismatched), the scalar `tolerance` applies to
    /// every index - the recorded `p99` suggestions from the analysis crate
    /// are written here.
    pub tolerances: Vec<PatternTolerance>,
    /// Original reference point positions, if stored in the profile.
    /// Required for partial matching when one contact pin is missing.
    pub ref_points: Option<Vec<Vec2<Millimeters>>>,
    /// Detection regions restricting where this figure may match
    /// (PROFILE_REGIONS.md). Empty = anywhere on the surface.
    pub regions: Vec<Region>,
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
            spawn_stability: FrameCount::new(15),
            num_points: PinCount::new(points.len()),
            sorted_distances: sorted_pairwise_distances(points.0),
            tolerance: tolerance.into(),
            tolerances: Vec::new(),
            ref_points: Some(points.to_vec()),
            regions: Vec::new(),
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
        let measured = sorted_pairwise_distances(points.0);
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
    pub fn match_score(&self, points: PointSet<'_>) -> f64 {
        self.score_of(&Self::deviations(
            &Self::distances_of(points),
            &self.sorted_distances,
        ))
    }

    /// Score of an already-computed deviation vector - `max_i dev[i] /
    /// tolerance_at(i)`; 0.0 for empty or length-mismatched input.
    pub fn score_of(&self, deviations: &[Millimeters]) -> f64 {
        deviations
            .iter()
            .enumerate()
            .map(|(i, dev)| dev.as_f64() / self.tolerance_at(i).as_f64())
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    /// Total absolute deviation across all distances (lower is better).
    pub fn match_error(&self, points: PointSet<'_>) -> Millimeters {
        Self::deviations(&Self::distances_of(points), &self.sorted_distances)
            .iter()
            .copied()
            .sum()
    }

    /// Computes the sorted pairwise distances of an arbitrary point set -
    /// the measured fingerprint compared against `sorted_distances`.
    pub fn distances_of(points: PointSet<'_>) -> Vec<Millimeters> {
        sorted_pairwise_distances(points.0)
    }

    /// Absolute per-index deviation between a measured and a reference
    /// fingerprint; empty when the lengths differ.
    pub fn deviations(measured: &[Millimeters], reference: &[Millimeters]) -> Vec<Millimeters> {
        if measured.len() != reference.len() {
            return Vec::new();
        }
        measured
            .iter()
            .zip(reference.iter())
            .map(|(m, r)| (*m - *r).abs())
            .collect()
    }

    /// Fingerprint of the reference point set with pin `skip` removed.
    ///
    /// Returns the kept reference indices and their sorted pairwise
    /// distances; `None` when no reference points are stored, their count
    /// differs from `num_points` or `skip` is out of range.
    pub fn partial_reference(&self, skip: usize) -> Option<PartialReference> {
        self.partial_reference_multi(&[skip])
    }

    /// Like [`Self::partial_reference`] with `skips` pins removed -
    /// needed for `num_points - 2` completion candidates.
    pub fn partial_reference_multi(&self, skips: &[usize]) -> Option<PartialReference> {
        let ref_points = self.ref_points.as_ref()?;
        if ref_points.len() != self.num_points
            || skips.iter().any(|skip| *skip >= self.num_points)
            || skips.is_empty()
        {
            return None;
        }
        let subset: Vec<Vec2<Millimeters>> = ref_points
            .iter()
            .enumerate()
            .filter(|(index, _)| !skips.contains(index))
            .map(|(_, point)| *point)
            .collect();
        let indices = (0..self.num_points.as_usize())
            .filter(|index| !skips.contains(index))
            .collect();
        Some(PartialReference {
            kept_indices: indices,
            distances: sorted_pairwise_distances(&subset),
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
        self.matches_partial_missing_scaled(points, missing, 1.0)
    }

    /// Like [`Self::matches_partial_missing`], with the partial tolerance
    /// scaled by `tol_scale` - incumbent re-matching relaxes the threshold
    /// (slot binding already proves identity); `1.0` keeps the default.
    pub fn matches_partial_missing_scaled(
        &self,
        points: PointSet<'_>,
        missing: usize,
        tol_scale: f64,
    ) -> Option<PartialMatch> {
        self.ref_points.as_ref()?;
        let partial_tolerance = self.tolerance.0 * 1.5 * tol_scale;
        let measured = sorted_pairwise_distances(points.0);
        let mut best: Option<PartialMatch> = None;

        for skips in skip_combinations(self.num_points.as_usize(), missing) {
            let Some(reference) = self.partial_reference_multi(&skips) else {
                continue;
            };

            let deviations = Self::deviations(&measured, &reference.distances);
            if deviations.len() == reference.distances.len()
                && deviations.iter().all(|d| *d <= partial_tolerance)
            {
                let error: Millimeters = deviations.iter().copied().sum();
                let candidate = PartialMatch {
                    kept_indices: reference.kept_indices,
                    match_error: error,
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
        kept_refs: &[usize],
        measured: PointSet<'_>,
        missing: usize,
    ) -> Option<Vec2<Millimeters>> {
        let ref_points = self.ref_points.as_ref()?;
        if missing >= self.num_points
            || kept_refs.len() != measured.len()
            || kept_refs.len() < 2
            || kept_refs.iter().any(|i| *i >= self.num_points)
        {
            return None;
        }
        let kept_ref_points: Vec<Vec2<Millimeters>> =
            kept_refs.iter().map(|&i| ref_points[i]).collect();
        // Try every correspondence consistent with the distance
        // multisets (symmetric figures admit several) and keep the
        // transform with the smallest residual.
        let mut best: Option<(RigidTransform, Millimeters)> = None;
        for mapping in correspondences(&kept_ref_points, measured.0, self.tolerance) {
            let Some(transform) = similarity_transform(&kept_ref_points, measured.0, &mapping)
            else {
                continue;
            };
            let residual: f64 = kept_ref_points
                .iter()
                .enumerate()
                .map(|(ri, r)| {
                    let measured_index =
                        mapping.iter().position(|&mapped| mapped == ri).unwrap_or(0);
                    transform
                        .apply(*r)
                        .distance(&measured[measured_index])
                        .as_f64()
                })
                .map(|d| d * d)
                .sum();
            let residual = Millimeters::new(residual.sqrt());
            if best.as_ref().is_none_or(|(_, r)| residual < *r) {
                best = Some((transform, residual));
            }
        }
        best.map(|(transform, _)| transform.apply(ref_points[missing]))
    }
}

/// Computes all pairwise distances of a point set, sorted ascending.
fn sorted_pairwise_distances(points: &[Vec2<Millimeters>]) -> Vec<Millimeters> {
    let mut distances = Vec::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            distances.push(points[i].distance(&points[j]));
        }
    }
    distances.sort_by(Millimeters::total_cmp);
    distances
}

/// All `missing`-element index combinations of `0..n` (the pins to skip).
fn skip_combinations(n: usize, missing: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    let mut combo = Vec::with_capacity(missing);
    fn go(
        n: usize,
        missing: usize,
        start: usize,
        combo: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if combo.len() == missing {
            out.push(combo.clone());
            return;
        }
        for i in start..n {
            combo.push(i);
            go(n, missing, i + 1, combo, out);
            combo.pop();
        }
    }
    go(n, missing, 0, &mut combo, &mut result);
    result
}

/// Sorted distance multiset of one point to all others in the set.
fn distance_multiset(points: &[Vec2<Millimeters>], index: usize) -> Vec<Millimeters> {
    let mut dists: Vec<Millimeters> = points
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != index)
        .map(|(_, other)| points[index].distance(other))
        .collect();
    dists.sort_by(Millimeters::total_cmp);
    dists
}

/// Enumerates every correspondence `measured[i] → refs[mapping[i]]`
/// consistent with per-point distance multisets (within `tolerance`).
/// Symmetric figures admit several mappings - the caller picks the one
/// whose fitted transform has the smallest residual.
fn correspondences(
    refs: &[Vec2<Millimeters>],
    measured: &[Vec2<Millimeters>],
    tolerance: PatternTolerance,
) -> Vec<Vec<usize>> {
    let tol = tolerance.as_millimeters();
    let ref_sets: Vec<Vec<Millimeters>> = (0..refs.len())
        .map(|i| distance_multiset(refs, i))
        .collect();
    let meas_sets: Vec<Vec<Millimeters>> = (0..measured.len())
        .map(|i| distance_multiset(measured, i))
        .collect();
    if refs.len() != measured.len() {
        return Vec::new();
    }

    // Candidates per measured point: ref indices with an equal multiset.
    let candidates: Vec<Vec<usize>> = meas_sets
        .iter()
        .map(|ms| {
            ref_sets
                .iter()
                .enumerate()
                .filter(|(_, rs)| {
                    ms.len() == rs.len()
                        && ms
                            .iter()
                            .zip(rs.iter())
                            .all(|(m, r)| (*m - *r).abs() <= tol)
                })
                .map(|(ri, _)| ri)
                .collect()
        })
        .collect();

    let mut result = Vec::new();
    let mut mapping = vec![usize::MAX; measured.len()];
    let mut used = vec![false; refs.len()];
    fn enumerate(
        candidates: &[Vec<usize>],
        index: usize,
        mapping: &mut Vec<usize>,
        used: &mut Vec<bool>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if index == candidates.len() {
            result.push(mapping.clone());
            return;
        }
        for &ri in &candidates[index] {
            if used[ri] {
                continue;
            }
            used[ri] = true;
            mapping[index] = ri;
            enumerate(candidates, index + 1, mapping, used, result);
            used[ri] = false;
        }
    }
    enumerate(&candidates, 0, &mut mapping, &mut used, &mut result);
    result
}

/// Rigid 2D transform (rotation + translation) fitted to the measured
/// points. `mapping[i]` maps `measured[i]` to `refs[mapping[i]]`.
struct RigidTransform {
    angle: f64,
    ref_center: Vec2<Millimeters>,
    meas_center: Vec2<Millimeters>,
}

impl RigidTransform {
    /// Applies the transform: `meas_center + R(angle) · (p - ref_center)`.
    fn apply(&self, p: Vec2<Millimeters>) -> Vec2<Millimeters> {
        let (sin, cos) = self.angle.sin_cos();
        let dx = p.x - self.ref_center.x;
        let dy = p.y - self.ref_center.y;
        Vec2::new(
            self.meas_center.x + dx * cos - dy * sin,
            self.meas_center.y + dx * sin + dy * cos,
        )
    }
}

/// Least-squares rigid transform: rotation angle via the summed
/// cross/dot products of centered points.
fn similarity_transform(
    refs: &[Vec2<Millimeters>],
    measured: &[Vec2<Millimeters>],
    mapping: &[usize],
) -> Option<RigidTransform> {
    let ref_center = Vec2::centroid(refs);
    let meas_center = Vec2::centroid(measured);
    let mut cross = 0.0;
    let mut dot = 0.0;
    for (mi, m) in measured.iter().enumerate() {
        let r = refs[mapping[mi]];
        let dr = (r.x - ref_center.x, r.y - ref_center.y);
        let dm = (m.x - meas_center.x, m.y - meas_center.y);
        cross += dr.0 * dm.1 - dr.1 * dm.0;
        dot += dr.0 * dm.0 + dr.1 * dm.1;
    }
    if dot.abs() + cross.abs() < 1e-9 {
        return None;
    }
    Some(RigidTransform {
        angle: cross.atan2(dot),
        ref_center,
        meas_center,
    })
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
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()];
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
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()];
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
        pattern.tolerances = vec![0.1.into()]; // too few - must not apply
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
        assert_eq!(pattern.match_score((&triangle()).into()), 0.0);
        // A 2.5 mm deviation on one edge is score 0.5 with scalar 5.0.
        let slightly_off = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(52.5, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let score = pattern.match_score((&slightly_off).into());
        assert!(score > 0.4 && score < 0.6);
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
