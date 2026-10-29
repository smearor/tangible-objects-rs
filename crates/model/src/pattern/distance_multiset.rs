use crate::Millimeters;
use crate::Vec2;

/// Sorted distance multiset of one point to all other points in its
/// set - the per-point fingerprint correspondence solving compares
/// (ROTATION.md). Two equal multisets mean the two points occupy the
/// same role in their respective geometries.
pub struct DistanceMultiset(pub Vec<Millimeters>);

impl DistanceMultiset {
    /// The multiset of `points[index]` - sorted distances to every
    /// other point in the set.
    pub fn of(points: &[Vec2<Millimeters>], index: usize) -> Self {
        let mut dists: Vec<Millimeters> = points
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != index)
            .map(|(_, other)| points[index].distance(other))
            .collect();
        dists.sort_by(Millimeters::total_cmp);
        Self(dists)
    }

    /// Whether `other` has an equal multiset - same length and every
    /// pairwise distance within `tolerance` of ours.
    pub fn matches_within(&self, other: &DistanceMultiset, tolerance: Millimeters) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(a, b)| (*a - *b).abs() <= tolerance)
    }
}
