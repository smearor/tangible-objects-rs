use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Stable identifier of a tangible instance.
///
/// Assigned by the engine's `Tracker` when a tangible is created
/// (`TangibleAdded`); distinguishes tangibles matching the same pattern and
/// survives across frames as long as the tangible is associated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TangibleId(pub u32);

impl TangibleId {
    /// Creates a tangible ID from its numeric value.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The raw numeric ID.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for TangibleId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<TangibleId> for u32 {
    fn from(id: TangibleId) -> Self {
        id.0
    }
}

impl Display for TangibleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
