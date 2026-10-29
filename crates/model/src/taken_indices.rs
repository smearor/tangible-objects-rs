use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

/// Touch indices already accounted for - incumbent slot-bindings
/// (`reserved`) plus indices consumed by earlier matches this frame
/// (TRACKING_STABILIZATION.md). The exclusion mask of a matching
/// pass; the claim pool draws from its complement.
#[derive(Debug, Clone, Default)]
pub struct TakenIndices(pub HashSet<usize>);

impl TakenIndices {
    /// The union of two taken sets - reserved bindings plus the
    /// indices consumed so far this pass.
    pub fn union(&self, other: &TakenIndices) -> TakenIndices {
        self.0.union(&other.0).copied().collect()
    }
}

impl FromIterator<usize> for TakenIndices {
    /// Collecting indices yields the taken set.
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<HashSet<usize>> for TakenIndices {
    /// A hash set IS a taken set.
    fn from(indices: HashSet<usize>) -> Self {
        Self(indices)
    }
}

impl Deref for TakenIndices {
    type Target = HashSet<usize>;

    /// Taken sets deref to the underlying `HashSet` - `contains`,
    /// `insert`, `extend`, `iter` stay available.
    fn deref(&self) -> &HashSet<usize> {
        &self.0
    }
}

impl DerefMut for TakenIndices {
    /// Mutable access to the underlying `HashSet`.
    fn deref_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.0
    }
}
