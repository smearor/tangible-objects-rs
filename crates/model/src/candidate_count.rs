use serde::Deserialize;
use serde::Serialize;
use std::ops::AddAssign;

/// A count of evaluated candidate combinations - a cardinality, not an
/// ordinal.
///
/// Generic matching enumerates k-subsets of the contact set per frame;
/// `candidates_tested` metrics count how many of them actually ran
/// through fingerprint evaluation (span pruning lowers it -
/// SPATIAL_CLUSTERING.md). `u64` because session-cumulative counts
/// exceed `u32` quickly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CandidateCount(pub u64);

impl CandidateCount {
    /// Creates a candidate count.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<usize> for CandidateCount {
    fn from(count: usize) -> Self {
        Self(count as u64)
    }
}

impl From<CandidateCount> for u64 {
    fn from(count: CandidateCount) -> Self {
        count.0
    }
}

impl AddAssign for CandidateCount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
