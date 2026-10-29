use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::AddAssign;

/// A count of [`super::CandidateReport`]s - a cardinality of tested point
/// combinations, not of frames.
///
/// One frame can hold several matched candidates of the same pattern (every
/// combination within tolerance reports `matched: true`; only the best is
/// adopted), so candidate counts and frame counts are distinct domains.
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

impl AddAssign<u64> for CandidateCount {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl AddAssign<CandidateCount> for CandidateCount {
    fn add_assign(&mut self, rhs: CandidateCount) {
        self.0 += rhs.0;
    }
}

impl From<u64> for CandidateCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl Display for CandidateCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
