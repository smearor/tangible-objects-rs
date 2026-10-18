use std::ops::Deref;

use crate::Millimeters;
use crate::PointSet;
use crate::Vec2;

/// Owned set of measured contact positions - the materialized
/// counterpart of the borrowed [`PointSet`] view.
///
/// Candidate matching materializes each combination's contact
/// positions into a `PointBuf`; everything that consumes positions
/// works on the `PointSet` view via [`Self::as_point_set`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointBuf(pub Vec<Vec2<Millimeters>>);

impl PointBuf {
    /// Borrowed view of this point set.
    pub fn as_point_set(&self) -> PointSet<'_> {
        PointSet::new(&self.0)
    }
}

impl From<Vec<Vec2<Millimeters>>> for PointBuf {
    /// A position vector IS an owned point set.
    fn from(points: Vec<Vec2<Millimeters>>) -> Self {
        Self(points)
    }
}

impl Deref for PointBuf {
    type Target = [Vec2<Millimeters>];

    /// Owned point sets deref to their position slice.
    fn deref(&self) -> &[Vec2<Millimeters>] {
        &self.0
    }
}
