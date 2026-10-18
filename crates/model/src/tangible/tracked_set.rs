use std::ops::Deref;

use super::TrackedTangible;

/// The currently alive tracked tangibles - a borrowed view, not an
/// allocation.
///
/// `Deref` keeps `len`/`iter`/indexing free; snapshot, metrics and
/// interface APIs take `TrackedSet` so "the tracked figures" is a
/// named concept, not an anonymous slice.
#[derive(Debug, Clone, Copy)]
pub struct TrackedSet<'a>(pub &'a [TrackedTangible]);

impl<'a> TrackedSet<'a> {
    /// Wraps a tracked-tangible slice.
    pub const fn new(tangibles: &'a [TrackedTangible]) -> Self {
        Self(tangibles)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [TrackedTangible] {
        self.0
    }
}

impl<'a> From<&'a [TrackedTangible]> for TrackedSet<'a> {
    fn from(tangibles: &'a [TrackedTangible]) -> Self {
        Self(tangibles)
    }
}

impl<'a> From<&'a Vec<TrackedTangible>> for TrackedSet<'a> {
    fn from(tangibles: &'a Vec<TrackedTangible>) -> Self {
        Self(tangibles)
    }
}

impl<'a> From<&'a super::TrackedBuf> for TrackedSet<'a> {
    /// Borrowed view of an owned tracked set.
    fn from(tangibles: &'a super::TrackedBuf) -> Self {
        Self(&tangibles.0)
    }
}

impl<'a, const N: usize> From<&'a [TrackedTangible; N]> for TrackedSet<'a> {
    fn from(tangibles: &'a [TrackedTangible; N]) -> Self {
        Self(tangibles)
    }
}

impl<'a> IntoIterator for TrackedSet<'a> {
    type Item = &'a TrackedTangible;
    type IntoIter = std::slice::Iter<'a, TrackedTangible>;

    /// Iterating a set is iterating its slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> Deref for TrackedSet<'a> {
    type Target = [TrackedTangible];

    /// Tracked sets view tangibles transparently.
    fn deref(&self) -> &[TrackedTangible] {
        self.0
    }
}
