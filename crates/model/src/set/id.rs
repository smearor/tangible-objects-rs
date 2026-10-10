use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Unique identifier of a Tangible Object Set (TOS).
///
/// Assigned when a set definition is loaded; used to key set state in the
/// [`crate::SetTracker`] and to correlate [`crate::SetAction`]s with their
/// set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TangibleSetId(pub u32);

impl TangibleSetId {
    /// Creates a set ID from its numeric value.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The raw numeric ID.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for TangibleSetId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<TangibleSetId> for u32 {
    fn from(id: TangibleSetId) -> Self {
        id.0
    }
}

impl Display for TangibleSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
