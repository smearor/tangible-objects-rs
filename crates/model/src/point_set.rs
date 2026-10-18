use std::ops::Deref;

use crate::Millimeters;
use crate::PointBuf;
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
