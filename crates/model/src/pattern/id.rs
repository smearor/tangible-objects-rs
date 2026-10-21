use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Runtime identifier of a figure pattern.
///
/// Assigned sequentially when a profile directory is loaded; used to
/// associate [`crate::PatternMatch`]es with [`crate::TrackedTangible`]s and
/// to pick UI colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct PatternId(pub u32);

impl PatternId {
    /// Creates a pattern ID from its numeric value.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The raw numeric ID.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for PatternId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<PatternId> for u32 {
    fn from(id: PatternId) -> Self {
        id.0
    }
}

impl Display for PatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(PatternId);

#[cfg(feature = "zbus")]
impl zvariant::Type for PatternId {
    const SIGNATURE: &'static zvariant::Signature = <u32 as zvariant::Type>::SIGNATURE;
}
