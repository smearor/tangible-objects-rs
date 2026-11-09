use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::AddAssign;

/// A count of occupancy fields - a cardinality, not an ordinal.
///
/// Used both for the total fields across all loaded [`FieldSet`]s
/// (`field_count`) and for the subset currently holding a confirmed
/// contact (`occupied_field_count`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldCount(pub u32);

impl FieldCount {
    /// Creates a field count.
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Whether the count is zero - no fields.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for FieldCount {
    fn from(count: u32) -> Self {
        Self(count)
    }
}

impl From<usize> for FieldCount {
    fn from(count: usize) -> Self {
        Self(count as u32)
    }
}

impl From<FieldCount> for u32 {
    fn from(count: FieldCount) -> Self {
        count.0
    }
}

impl AddAssign<FieldCount> for FieldCount {
    fn add_assign(&mut self, rhs: FieldCount) {
        self.0 += rhs.0;
    }
}

impl Display for FieldCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(FieldCount);
