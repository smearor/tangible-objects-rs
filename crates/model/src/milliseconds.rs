use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::time::Duration;

/// A duration or timestamp in milliseconds.
///
/// Unit counterpart to [`crate::Millimeters`]; used for frame timestamps in
/// recordings (`timestamp_ms`). Wraps the plain `u64` so timestamps cannot be
/// confused with frame numbers or millimeter values. Serializes as a bare
/// number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Milliseconds(pub u64);

impl Milliseconds {
    /// Creates a millisecond value.
    pub const fn new(milliseconds: u64) -> Self {
        Self(milliseconds)
    }

    /// The numeric value in milliseconds.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for Milliseconds {
    fn from(milliseconds: u64) -> Self {
        Self(milliseconds)
    }
}

impl From<Milliseconds> for u64 {
    fn from(milliseconds: Milliseconds) -> Self {
        milliseconds.0
    }
}

impl From<Duration> for Milliseconds {
    fn from(duration: Duration) -> Self {
        Self(duration.as_millis() as u64)
    }
}

impl From<Milliseconds> for Duration {
    fn from(milliseconds: Milliseconds) -> Self {
        Duration::from_millis(milliseconds.0)
    }
}

impl Display for Milliseconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}
