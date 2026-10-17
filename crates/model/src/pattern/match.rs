use super::PatternId;
use crate::Millimeters;
use crate::RotationRad;
use crate::Vec2;

/// A successful pattern match within a single input frame.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// ID of the matched pattern (assigned at profile load time).
    pub pattern_id: PatternId,
    /// Name of the matched figure profile.
    pub pattern_name: String,
    /// Centroid of the matched touch points.
    pub center: Vec2<Millimeters>,
    /// Estimated figure orientation in radians.
    pub rotation_rad: RotationRad,
    /// Indices into the frame's touch point list that belong to this figure.
    pub touch_indices: Vec<usize>,
    /// Whether the match used a partial (`num_points - k`) fingerprint.
    /// `true` also for completed partials - see `claimed_indices`.
    pub partial: bool,
    /// Touch indices claimed by missing-pin completion - pins predicted
    /// from the matched geometry and assigned to this figure even though
    /// they did not satisfy the fingerprint (TRACKING_STABILIZATION.md).
    /// Empty when completion did not run or found nothing.
    pub claimed_indices: Vec<usize>,
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
            .chain(self.claimed_indices.iter())
            .copied()
    }

    /// Computes the figure orientation from its matched points.
    ///
    /// For 3 points the vertex opposite the shortest side defines the
    /// orientation; for 4+ points the point farthest from the centroid is used.
    /// Returns 0.0 for empty point sets.
    pub fn rotation_of(points: &[Vec2<Millimeters>], center: &Vec2<Millimeters>) -> RotationRad {
        if points.is_empty() {
            return RotationRad::new(0.0);
        }
        if points.len() == 3 {
            let d01 = points[0].distance(&points[1]);
            let d12 = points[1].distance(&points[2]);
            let d02 = points[0].distance(&points[2]);

            let ref_index = if d01 <= d12 && d01 <= d02 {
                2 // side 0-1 shortest, opposite vertex is 2
            } else if d12 <= d01 && d12 <= d02 {
                0 // side 1-2 shortest, opposite vertex is 0
            } else {
                1 // side 0-2 shortest, opposite vertex is 1
            };

            let p = &points[ref_index];
            RotationRad::new((p.y - center.y).atan2(p.x - center.x))
        } else {
            let farthest = points
                .iter()
                .max_by(|a, b| a.distance(center).total_cmp(&b.distance(center)));
            match farthest {
                Some(p) => RotationRad::new((p.y - center.y).atan2(p.x - center.x)),
                None => RotationRad::new(0.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PatternMatch;
    use crate::Vec2;

    #[test]
    fn rotation_of_triangle_uses_opposite_vertex() {
        // Shortest side is 0-2 (length 30), so vertex 1 defines orientation.
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 50.0),
            Vec2::new(30.0, 0.0),
        ];
        let center = Vec2::centroid(&points);
        let rotation = PatternMatch::rotation_of(&points, &center).as_f64();
        // Vertex 1 at (0, 50), center at (10, 50/3): direction points up-left.
        assert!(rotation > std::f64::consts::PI / 2.0);
    }

    #[test]
    fn rotation_of_four_points_uses_farthest() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(100.0, 0.0),
        ];
        let center = Vec2::centroid(&points);
        let rotation = PatternMatch::rotation_of(&points, &center).as_f64();
        // Farthest point (100, 0) lies to the right of the centroid.
        assert!(rotation.abs() < std::f64::consts::PI / 2.0);
    }

    #[test]
    fn rotation_of_empty_set_is_zero() {
        assert_eq!(
            PatternMatch::rotation_of(&[], &Vec2::new(0.0, 0.0)).as_f64(),
            0.0
        );
    }

    #[test]
    fn partial_flag_and_all_indices() {
        let mut pattern_match = PatternMatch {
            pattern_id: crate::PatternId::new(1),
            pattern_name: "Figur8".to_string(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: crate::RotationRad::new(0.0),
            touch_indices: vec![0, 1, 2],
            partial: false,
            claimed_indices: Vec::new(),
        };
        assert!(!pattern_match.is_partial());
        assert_eq!(pattern_match.all_indices().count(), 3);
        pattern_match.partial = true;
        pattern_match.claimed_indices = vec![5];
        assert!(pattern_match.is_partial());
        assert_eq!(
            pattern_match.all_indices().collect::<Vec<usize>>(),
            vec![0, 1, 2, 5]
        );
    }
}
