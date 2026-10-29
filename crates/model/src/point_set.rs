use std::ops::Deref;

use crate::Millimeters;
use crate::PointBuf;
use crate::RotationRad;
use crate::Vec2;

/// Measured contact positions of one frame - a borrowed view, not an
/// allocation.
///
/// Distinguishes *measured* positions from the *reference* pin positions
/// stored in a pattern: matching APIs take `PointSet`, profiles store
/// `Vec<Vec2<Millimeters>>`. `Deref` keeps `len`/`iter`/indexing free.
#[derive(Debug, Clone, Copy)]
pub struct PointSet<'a>(pub &'a [Vec2<Millimeters>]);

impl<'a> PointSet<'a> {
    /// Wraps a contact-position slice.
    pub const fn new(points: &'a [Vec2<Millimeters>]) -> Self {
        Self(points)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [Vec2<Millimeters>] {
        self.0
    }

    /// Sorted pairwise distances of this point set - the measured
    /// fingerprint compared against `pattern.sorted_distances`.
    pub fn distances(self) -> crate::Distances {
        crate::Distances::of(self)
    }

    /// The point farthest from `center` - the orientation anchor of a
    /// symmetric footprint's argmax fallback (ROTATION.md): a figure's
    /// heading reads off the contact most distant from its centroid.
    pub fn farthest_from(self, center: &Vec2<Millimeters>) -> Option<Vec2<Millimeters>> {
        self.0
            .iter()
            .max_by(|a, b| a.distance(center).total_cmp(&b.distance(center)))
            .copied()
    }

    /// Estimated heading of this point set around `center` - the
    /// argmax orientation estimate before a correspondence solve
    /// (ROTATION.md): for 3 points the vertex opposite the shortest
    /// side defines the orientation; for 4+ points the point farthest
    /// from the centroid is used. `0.0` for empty point sets.
    pub fn rotation_of(self, center: &Vec2<Millimeters>) -> RotationRad {
        if self.is_empty() {
            return RotationRad::new(0.0);
        }
        if self.len() == 3 {
            let d01 = self[0].distance(&self[1]);
            let d12 = self[1].distance(&self[2]);
            let d02 = self[0].distance(&self[2]);

            let ref_index = if d01 <= d12 && d01 <= d02 {
                2 // side 0-1 shortest, opposite vertex is 2
            } else if d12 <= d01 && d12 <= d02 {
                0 // side 1-2 shortest, opposite vertex is 0
            } else {
                1 // side 0-2 shortest, opposite vertex is 1
            };

            let p = &self[ref_index];
            RotationRad::new((p.y - center.y).atan2(p.x - center.x))
        } else {
            match self.farthest_from(center) {
                Some(p) => RotationRad::new((p.y - center.y).atan2(p.x - center.x)),
                None => RotationRad::new(0.0),
            }
        }
    }
}

impl<'a> IntoIterator for PointSet<'a> {
    type Item = &'a Vec2<Millimeters>;
    type IntoIter = std::slice::Iter<'a, Vec2<Millimeters>>;

    /// Iterating a set is iterating its slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> From<&'a [Vec2<Millimeters>]> for PointSet<'a> {
    fn from(points: &'a [Vec2<Millimeters>]) -> Self {
        Self(points)
    }
}

impl<'a, const N: usize> From<&'a [Vec2<Millimeters>; N]> for PointSet<'a> {
    fn from(points: &'a [Vec2<Millimeters>; N]) -> Self {
        Self(points)
    }
}

impl<'a> From<&'a Vec<Vec2<Millimeters>>> for PointSet<'a> {
    fn from(points: &'a Vec<Vec2<Millimeters>>) -> Self {
        Self(points)
    }
}

impl<'a> From<&'a PointBuf> for PointSet<'a> {
    /// Borrowed view of an owned point set.
    fn from(points: &'a PointBuf) -> Self {
        Self(&points.0)
    }
}

impl<'a> Deref for PointSet<'a> {
    type Target = [Vec2<Millimeters>];

    /// Point sets view positions transparently.
    fn deref(&self) -> &[Vec2<Millimeters>] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PointSet;
    use crate::Millimeters;
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
        let set: PointSet<'_> = (&points).into();
        let rotation = set.rotation_of(&center).as_f64();
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
        let set: PointSet<'_> = (&points).into();
        let rotation = set.rotation_of(&center).as_f64();
        // Farthest point (100, 0) lies to the right of the centroid.
        assert!(rotation.abs() < std::f64::consts::PI / 2.0);
    }

    #[test]
    fn rotation_of_empty_set_is_zero() {
        let empty: &[Vec2<Millimeters>] = &[];
        let set: PointSet<'_> = empty.into();
        assert_eq!(set.rotation_of(&Vec2::new(0.0, 0.0)).as_f64(), 0.0);
    }
}
