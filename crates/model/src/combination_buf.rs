use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use crate::Combination;

/// Owned touch indices of a winning candidate - the materialized
/// counterpart of the borrowed [`Combination`] view.
///
/// `PatternMatch` and `CandidateReport` store the indices that formed
/// the figure; matching APIs work on the `Combination` view.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CombinationBuf(pub Vec<usize>);

impl CombinationBuf {
    /// Borrowed view of this combination.
    pub fn as_combination(&self) -> Combination<'_> {
        Combination::new(&self.0)
    }
}

impl<'a> From<Combination<'a>> for CombinationBuf {
    /// Materializes a borrowed combination.
    fn from(combo: Combination<'a>) -> Self {
        Self(combo.as_slice().to_vec())
    }
}

impl From<Vec<usize>> for CombinationBuf {
    /// An index vector IS an owned combination.
    fn from(indices: Vec<usize>) -> Self {
        Self(indices)
    }
}

impl From<&[usize]> for CombinationBuf {
    /// Materializes an index slice.
    fn from(indices: &[usize]) -> Self {
        Self(indices.to_vec())
    }
}

impl<'a> IntoIterator for &'a CombinationBuf {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    /// Iterating a buffer is iterating its indices.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for CombinationBuf {
    type Target = [usize];

    /// Owned combinations deref to their index slice.
    fn deref(&self) -> &[usize] {
        &self.0
    }
}
