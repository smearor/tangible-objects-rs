use std::ops::Deref;
use std::ops::DerefMut;

use serde::Serialize;

use super::TrackedSet;
use super::TrackedTangible;

/// Owned set of alive tracked tangibles - the materialized
/// counterpart of the borrowed [`TrackedSet`] view.
///
/// The tracker and snapshots store a `TrackedBuf`; everything
/// consuming the tracked set works on the `TrackedSet` view via
/// [`Self::as_tracked_set`].
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct TrackedBuf(pub Vec<TrackedTangible>);

impl TrackedBuf {
    /// Borrowed view of this tracked set.
    pub fn as_tracked_set(&self) -> TrackedSet<'_> {
        TrackedSet::new(&self.0)
    }
}

impl From<Vec<TrackedTangible>> for TrackedBuf {
    /// A tangible vector IS an owned tracked set.
    fn from(tangibles: Vec<TrackedTangible>) -> Self {
        Self(tangibles)
    }
}

impl From<TrackedBuf> for Vec<TrackedTangible> {
    /// Unwraps the owned tracked vector.
    fn from(tangibles: TrackedBuf) -> Self {
        tangibles.0
    }
}

impl Deref for TrackedBuf {
    type Target = Vec<TrackedTangible>;

    /// Owned tracked sets deref to their vector - `push`/`retain` for
    /// the tracker lifecycle lives on the owned side.
    fn deref(&self) -> &Vec<TrackedTangible> {
        &self.0
    }
}

impl DerefMut for TrackedBuf {
    /// Mutable access adds or retires tracked tangibles.
    fn deref_mut(&mut self) -> &mut Vec<TrackedTangible> {
        &mut self.0
    }
}
