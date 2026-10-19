use std::ops::Deref;

use super::TouchPoint;
use crate::PointBuf;

/// The active contacts of one frame - a borrowed view, not an
/// allocation.
///
/// `Deref` keeps `len`/`iter`/indexing free; matching, tracking and
/// field APIs take `TouchSet` so "the frame's contacts" is a named
/// concept, not an anonymous slice.
#[derive(Debug, Clone, Copy)]
pub struct TouchSet<'a>(pub &'a [TouchPoint]);

impl<'a> TouchSet<'a> {
    /// The empty contact set.
    pub const EMPTY: Self = Self(&[]);

    /// Wraps a contact slice.
    pub const fn new(touches: &'a [TouchPoint]) -> Self {
        Self(touches)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [TouchPoint] {
        self.0
    }
}

impl Default for TouchSet<'_> {
    /// The empty contact set - same as [`TouchSet::EMPTY`].
    fn default() -> Self {
        Self::EMPTY
    }
}

impl TouchSet<'_> {
    /// Collects the positions of the `indices`-selected contacts - the
    /// measured point set a candidate combination refers to.
    pub fn positions_of(self, indices: &[usize]) -> PointBuf {
        PointBuf(indices.iter().map(|&i| self.0[i].pos).collect())
    }
}

impl<'a> IntoIterator for TouchSet<'a> {
    type Item = &'a TouchPoint;
    type IntoIter = std::slice::Iter<'a, TouchPoint>;

    /// Iterating a set is iterating its slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> From<&'a [TouchPoint]> for TouchSet<'a> {
    fn from(touches: &'a [TouchPoint]) -> Self {
        Self(touches)
    }
}

impl<'a, const N: usize> From<&'a [TouchPoint; N]> for TouchSet<'a> {
    fn from(touches: &'a [TouchPoint; N]) -> Self {
        Self(touches)
    }
}

impl<'a> From<&'a Vec<TouchPoint>> for TouchSet<'a> {
    fn from(touches: &'a Vec<TouchPoint>) -> Self {
        Self(touches)
    }
}

impl<'a> Deref for TouchSet<'a> {
    type Target = [TouchPoint];

    /// Touch sets view contacts transparently.
    fn deref(&self) -> &[TouchPoint] {
        self.0
    }
}
