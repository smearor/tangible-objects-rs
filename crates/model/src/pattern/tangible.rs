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
    pub sorted_distances: Vec<f64>,
    /// Maximum allowed deviation per distance during matching, in mm.
    pub tolerance: PatternTolerance,
    /// Original reference point positions in mm, if stored in the profile.
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
            ref_points: Some(points.to_vec()),
        }
    }

    /// Checks whether a set of measured points matches this pattern exactly.
    pub fn matches(&self, points: &[Vec2<Millimeters>]) -> bool {
        if points.len() != self.num_points {
            return false;
        }
        let measured = sorted_pairwise_distances(points);
        measured.len() == self.sorted_distances.len()
            && measured
                .iter()
                .zip(self.sorted_distances.iter())
                .all(|(m, t)| (m - t).abs() <= self.tolerance.0)
    }

    /// Total absolute deviation across all distances (lower is better).
    pub fn match_error(&self, points: &[Vec2<Millimeters>]) -> f64 {
        let measured = sorted_pairwise_distances(points);
        measured
            .iter()
            .zip(self.sorted_distances.iter())
            .map(|(m, t)| (m - t).abs())
            .sum()
    }

    /// Partial match: tests whether `points` (`num_points - 1` contacts) fit a
    /// subset of the reference points with one pin missing (e.g. a tilting
    /// object). Every possibility of dropping one reference point is tried.
    /// The tolerance is raised by 50% to account for the higher uncertainty.
    ///
    /// Returns the matched reference indices and the total error.
    pub fn matches_partial(&self, points: &[Vec2<Millimeters>]) -> Option<(Vec<usize>, f64)> {
        let ref_pts = self.ref_points.as_ref()?;
        if points.len() != self.num_points - 1 || ref_pts.len() != self.num_points {
            return None;
        }
        let partial_tolerance = self.tolerance.0 * 1.5;
        let measured = sorted_pairwise_distances(points);
        let mut best: Option<(Vec<usize>, f64)> = None;

        for skip in 0..self.num_points {
            let subset: Vec<&Vec2<Millimeters>> = ref_pts
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, p)| p)
                .collect();

            let mut ref_dists = Vec::new();
            for i in 0..subset.len() {
                for j in (i + 1)..subset.len() {
                    ref_dists.push(subset[i].distance(subset[j]));
                }
            }
            ref_dists.sort_by(f64::total_cmp);

            if measured.len() == ref_dists.len()
                && measured
                    .iter()
                    .zip(ref_dists.iter())
                    .all(|(m, t)| (m - t).abs() <= partial_tolerance)
            {
                let error: f64 = measured
                    .iter()
                    .zip(ref_dists.iter())
                    .map(|(m, t)| (m - t).abs())
                    .sum();
                let all_indices: Vec<usize> = (0..self.num_points).filter(|i| *i != skip).collect();
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
fn sorted_pairwise_distances(points: &[Vec2<Millimeters>]) -> Vec<f64> {
    let mut distances = Vec::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            distances.push(points[i].distance(&points[j]));
        }
    }
    distances.sort_by(f64::total_cmp);
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
