use std::ops::Deref;
use std::ops::DerefMut;

use crate::CombinationBuf;

/// The enumerated positional mappings of a correspondence solve -
/// every `measured[i] → ref[mapping[i]]` assignment consistent with
/// the distance-multiset fingerprint (ROTATION.md).
///
/// Distinct from [`crate::Combinations`]: a combination is a *subset*
/// of indices, a correspondence is a *permutation* - every position
/// assigned, order significant.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Correspondences(pub Vec<CombinationBuf>);

impl IntoIterator for Correspondences {
    type Item = CombinationBuf;
    type IntoIter = std::vec::IntoIter<CombinationBuf>;

    /// Iterating enumerates the owned mappings.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Correspondences {
    type Item = &'a CombinationBuf;
    type IntoIter = std::slice::Iter<'a, CombinationBuf>;

    /// Borrowed iteration over the mappings.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<CombinationBuf>> for Correspondences {
    /// A mapping list IS the correspondences set.
    fn from(mappings: Vec<CombinationBuf>) -> Self {
        Self(mappings)
    }
}

impl Deref for Correspondences {
    type Target = Vec<CombinationBuf>;

    /// Correspondences view their vector transparently.
    fn deref(&self) -> &Vec<CombinationBuf> {
        &self.0
    }
}

impl DerefMut for Correspondences {
    /// Mutation during enumeration.
    fn deref_mut(&mut self) -> &mut Vec<CombinationBuf> {
        &mut self.0
    }
}
