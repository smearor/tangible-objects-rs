use std::fmt::Display;
use std::ops::AddAssign;

use serde::Deserialize;
use serde::Serialize;

/// Count of raw evdev events — e.g. events processed or dropped.
///
/// Keeps event cardinalities unconfusable with frame counts
/// ([`crate::FrameCount`]) and other `u64` quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventCount(pub u64);

impl EventCount {
    /// Creates an event count.
    pub fn new(count: u64) -> Self {
        Self(count)
    }

    /// The inner count.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl AddAssign<u64> for EventCount {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl From<u64> for EventCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl Display for EventCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(EventCount);
