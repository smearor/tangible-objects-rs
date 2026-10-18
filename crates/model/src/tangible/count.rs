use serde::Deserialize;
use serde::Serialize;

/// A count of distinct tracked tangibles - a cardinality, not an ID.
///
/// Used for session metrics ("how many distinct tangible identities
/// were spawned") where the plural matters less than the quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TangibleCount(pub u64);

impl TangibleCount {
    /// Creates a tangible count.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for TangibleCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl From<TangibleCount> for u64 {
    fn from(count: TangibleCount) -> Self {
        count.0
    }
}
