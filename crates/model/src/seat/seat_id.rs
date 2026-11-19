use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

/// Runtime identifier of a [`crate::TableTopSeat`].
///
/// Assigned sequentially by the [`crate::SeatTracker`] when a seat is
/// added to a session; never reused. Unique across all sessions, so
/// interfaces address seats by ID alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct SeatId(pub u32);

impl SeatId {
    /// Creates a seat ID from its numeric value.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// The ID following this one - the successor the sequential seat
    /// counter assigns next.
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// The raw numeric ID.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for SeatId {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

impl From<SeatId> for u32 {
    fn from(id: SeatId) -> Self {
        id.0
    }
}

impl FromStr for SeatId {
    type Err = ParseIntError;

    /// Parses the numeric seat ID (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Self::new)
    }
}

impl Display for SeatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SeatId);

#[cfg(feature = "zbus")]
impl zvariant::Type for SeatId {
    const SIGNATURE: &'static zvariant::Signature = <u32 as zvariant::Type>::SIGNATURE;
}
