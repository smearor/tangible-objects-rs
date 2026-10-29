use std::ops::Deref;

use crate::ClaimPool;
use crate::Combination;

/// Touch indices a matcher or claimer may still pick from - the
/// candidate pool of a matching pass (TRACKING_STABILIZATION.md).
///
/// The complement of [`crate::TakenIndices`]: `available` holds the
/// positions `combinations` enumerates over; `free` is the claim-pool
/// remainder after a combination took its positions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CandidateIndices(pub Vec<usize>);

impl CandidateIndices {
    /// The claim-pool view of these candidates - what `claim_missing`
    /// searches (the `CombinationBuf::as_combination` parallel).
    pub fn as_claim_pool(&self) -> ClaimPool<'_> {
        ClaimPool(&self.0)
    }

    /// The pool with `combo`'s positions removed - the claim search
    /// space a partial match's `claim_missing` draws from.
    pub fn without(&self, combo: Combination<'_>) -> CandidateIndices {
        self.iter()
            .copied()
            .filter(|index| !combo.contains(index))
            .collect()
    }
}

impl From<Vec<usize>> for CandidateIndices {
    /// An index vector IS a candidate pool.
    fn from(indices: Vec<usize>) -> Self {
        Self(indices)
    }
}

impl FromIterator<usize> for CandidateIndices {
    /// Collecting indices yields a candidate pool.
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for CandidateIndices {
    type Target = [usize];

    /// Candidate pools deref to their index slice - `len`, `contains`,
    /// `iter` and `&[usize]`-taking call sites stay available.
    fn deref(&self) -> &[usize] {
        &self.0
    }
}
