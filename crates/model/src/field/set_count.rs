use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// A count of [`FieldSet`]s - a cardinality, not an ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldSetCount(pub u32);

impl FieldSetCount {
    /// Creates a field-set count.
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Whether the count is zero - no field sets.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for FieldSetCount {
    fn from(count: u32) -> Self {
        Self(count)
    }
}

impl From<usize> for FieldSetCount {
    fn from(count: usize) -> Self {
        Self(count as u32)
    }
}

impl From<FieldSetCount> for u32 {
    fn from(count: FieldSetCount) -> Self {
        count.0
    }
}

impl Display for FieldSetCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(FieldSetCount);
