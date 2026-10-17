use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::AddAssign;

/// A count of touch contacts - a cardinality, not an ordinal.
///
/// Used for occupancy fields ("how many contacts are inside this cell")
/// and anywhere a number of simultaneous contacts is meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContactCount(pub u32);

impl ContactCount {
    /// Creates a contact count.
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Whether the count is zero - the field is empty.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for ContactCount {
    fn from(count: u32) -> Self {
        Self(count)
    }
}

impl From<usize> for ContactCount {
    fn from(count: usize) -> Self {
        Self(count as u32)
    }
}

impl From<ContactCount> for u32 {
    fn from(count: ContactCount) -> Self {
        count.0
    }
}

impl AddAssign<ContactCount> for ContactCount {
    fn add_assign(&mut self, rhs: ContactCount) {
        self.0 += rhs.0;
    }
}

impl Display for ContactCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(ContactCount);
