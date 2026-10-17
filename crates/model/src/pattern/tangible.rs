use super::PatternId;
use super::PatternTolerance;
use crate::Millimeters;
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
    pub name: String,
    /// Number of contact pins the figure must have.
    pub num_points: usize,
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
}

impl TangiblePattern {
    /// Creates a pattern from known reference point positions.
    pub fn from_points(
        id: impl Into<PatternId>,
        name: &str,
        points: &[Vec2<Millimeters>],
        tolerance: impl Into<PatternTolerance>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.to_string(),
            num_points: points.len(),
            sorted_distances: sorted_pairwise_distances(points),
            tolerance: tolerance.into(),
            tolerances: Vec::new(),
            ref_points: Some(points.to_vec()),
        }
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
    pub fn matches(&self, points: &[Vec2<Millimeters>]) -> bool {
        if points.len() != self.num_points {
            return false;
        }
        let measured = sorted_pairwise_distances(points);
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
    pub fn match_score(&self, points: &[Vec2<Millimeters>]) -> f64 {
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
    pub fn match_error(&self, points: &[Vec2<Millimeters>]) -> Millimeters {
        Self::deviations(&Self::distances_of(points), &self.sorted_distances)
            .iter()
            .copied()
            .sum()
    }

    /// Computes the sorted pairwise distances of an arbitrary point set -
    /// the measured fingerprint compared against `sorted_distances`.
    pub fn distances_of(points: &[Vec2<Millimeters>]) -> Vec<Millimeters> {
        sorted_pairwise_distances(points)
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
    pub fn partial_reference(&self, skip: usize) -> Option<(Vec<usize>, Vec<Millimeters>)> {
        let ref_points = self.ref_points.as_ref()?;
        if ref_points.len() != self.num_points || skip >= self.num_points {
            return None;
        }
        let subset: Vec<Vec2<Millimeters>> = ref_points
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != skip)
            .map(|(_, point)| *point)
            .collect();
        let indices = (0..self.num_points)
            .filter(|index| *index != skip)
            .collect();
        Some((indices, sorted_pairwise_distances(&subset)))
    }

    /// Partial match: tests whether `points` (`num_points - 1` contacts) fit a
    /// subset of the reference points with one pin missing (e.g. a tilting
    /// object). Every possibility of dropping one reference point is tried.
    /// The scalar tolerance is raised by 50% to account for the higher
    /// uncertainty - per-index `tolerances` cannot apply here because
    /// dropping a pin re-indexes the distance fingerprint.
    ///
    /// Returns the matched reference indices and the total error.
    pub fn matches_partial(
        &self,
        points: &[Vec2<Millimeters>],
    ) -> Option<(Vec<usize>, Millimeters)> {
        let ref_pts = self.ref_points.as_ref()?;
        if self.num_points == 0
            || points.len() != self.num_points - 1
            || ref_pts.len() != self.num_points
        {
            return None;
        }
        let partial_tolerance = self.tolerance.0 * 1.5;
        let measured = sorted_pairwise_distances(points);
        let mut best: Option<(Vec<usize>, Millimeters)> = None;

        for skip in 0..self.num_points {
            let Some((all_indices, ref_dists)) = self.partial_reference(skip) else {
                continue;
            };

            let deviations = Self::deviations(&measured, &ref_dists);
            if deviations.len() == ref_dists.len()
                && deviations.iter().all(|d| *d <= partial_tolerance)
            {
                let error: Millimeters = deviations.iter().copied().sum();
                match &best {
                    None => best = Some((all_indices, error)),
                    Some((_, best_err)) if error < *best_err => best = Some((all_indices, error)),
                    _ => {}
                }
            }
        }
        best
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
        let pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        assert_eq!(pattern.sorted_distances.len(), 3);
    }

    #[test]
    fn same_geometry_different_pose_matches() {
        let pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        // Same triangle, translated and rotated.
        let rotated = vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 150.0),
            Vec2::new(150.0, 100.0),
        ];
        assert!(pattern.matches(&rotated));
    }

    #[test]
    fn wrong_geometry_does_not_match() {
        let pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        let other = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(0.0, 60.0),
        ];
        assert!(!pattern.matches(&other));
    }

    #[test]
    fn wrong_point_count_does_not_match() {
        let pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        assert!(!pattern.matches(&triangle()[..2]));
    }

    #[test]
    fn partial_match_with_missing_pin() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let pattern = TangiblePattern::from_points(1, "square", &square, 5.0);
        assert!(pattern.matches_partial(&square[..3]).is_some());
    }

    #[test]
    fn per_index_tolerances_refine_scalar() {
        // Fingerprint [50, 50, ~70.7]; indices 1–2 generous, index 0 strict.
        let mut pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 1.0);
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()];
        // Stretched third contact: deviations [0, ~15, ~11] - the shortest
        // edge is exact, the longer ones sit inside their 20 mm budget.
        // Rejected under scalar 1.0, accepted per-index.
        let stretched = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 65.0),
        ];
        let scalar_only = TangiblePattern::from_points(1, "triangle", &triangle(), 1.0);
        assert!(!scalar_only.matches(&stretched));
        assert!(pattern.matches(&stretched));
    }

    #[test]
    fn per_index_tolerances_still_reject_short_edges() {
        let mut pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 1.0);
        pattern.tolerances = vec![1.0.into(), 20.0.into(), 20.0.into()];
        // The shortest edge shrinks (45 vs 50): index 0 deviates by 5 mm and
        // exceeds its strict 1 mm budget even though indices 1–2 would allow
        // the deformation.
        let warped = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(45.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        assert!(!pattern.matches(&warped));
    }

    #[test]
    fn mismatched_tolerances_length_falls_back_to_scalar() {
        let mut pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        pattern.tolerances = vec![0.1.into()]; // too few - must not apply
        let slightly_off = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(53.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        assert!(pattern.matches(&slightly_off)); // scalar 5.0 still applies
    }

    #[test]
    fn match_score_marks_boundary_and_near_misses() {
        let pattern = TangiblePattern::from_points(1, "triangle", &triangle(), 5.0);
        assert_eq!(pattern.match_score(&triangle()), 0.0);
        // A 2.5 mm deviation on one edge is score 0.5 with scalar 5.0.
        let slightly_off = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(52.5, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let score = pattern.match_score(&slightly_off);
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
        let mut pattern = TangiblePattern::from_points(1, "square", &square, 5.0);
        pattern.ref_points = None;
        assert!(pattern.matches_partial(&square[..3]).is_none());
    }
}
