use std::ops::Deref;
use std::ops::DerefMut;

use crate::CombinationBuf;

/// The materialized set of all candidate combinations - every
/// k-element index subset of an index range.
///
/// Matching enumerates one [`crate::Combination`] per contained
/// [`CombinationBuf`]; the inner vectors are candidates, the outer
/// collection is the enumeration itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Combinations(pub Vec<CombinationBuf>);

impl Combinations {
    /// An empty combination set.
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of combinations in the set - C(n, k).
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the set is empty (n < k or empty input).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The single empty combination - the C(n, 0) base case of
    /// combination enumeration.
    pub fn one_empty() -> Self {
        Self(vec![CombinationBuf(Vec::new())])
    }
}

impl From<Vec<CombinationBuf>> for Combinations {
    /// A combination list IS a combinations set.
    fn from(combinations: Vec<CombinationBuf>) -> Self {
        Self(combinations)
    }
}

impl FromIterator<Vec<usize>> for Combinations {
    /// Collecting index vectors yields the combinations set.
    fn from_iter<T: IntoIterator<Item = Vec<usize>>>(iter: T) -> Self {
        Self(iter.into_iter().map(CombinationBuf).collect())
    }
}

impl IntoIterator for Combinations {
    type Item = CombinationBuf;
    type IntoIter = std::vec::IntoIter<CombinationBuf>;

    /// Iterating enumerates the owned combinations.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Combinations {
    type Item = &'a CombinationBuf;
    type IntoIter = std::slice::Iter<'a, CombinationBuf>;

    /// Borrowed iteration over the combinations.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Combinations {
    type Target = Vec<CombinationBuf>;

    /// Combination sets view their vector transparently.
    fn deref(&self) -> &Vec<CombinationBuf> {
        &self.0
    }
}

impl DerefMut for Combinations {
    /// Mutation builds the set - `push`/`extend` during enumeration.
    fn deref_mut(&mut self) -> &mut Vec<CombinationBuf> {
        &mut self.0
    }
}
