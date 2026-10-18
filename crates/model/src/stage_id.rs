use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Runtime identifier of an event-pipeline stage.
///
/// Assigned when a stage is registered on the pipeline; never reused so a
/// stale `StageId` can never alias a newer stage (same rule as
/// [`crate::PatternId`]). Keys `StageDescriptor`s, per-stage metrics and
/// the `stage` scope on profiles - introduced by EVENT_PIPELINE.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StageId(pub u32);

impl StageId {
    /// Creates a stage ID from its numeric value.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The raw numeric ID.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for StageId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<StageId> for u32 {
    fn from(id: StageId) -> Self {
        id.0
    }
}

impl Display for StageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(StageId);

#[cfg(feature = "zbus")]
impl zvariant::Type for StageId {
    const SIGNATURE: &'static zvariant::Signature = <u32 as zvariant::Type>::SIGNATURE;
}
