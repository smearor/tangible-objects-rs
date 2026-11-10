//! `FingerCount` - how many simultaneous contacts a spell's gesture
//! uses.

use serde::Deserialize;
use serde::Serialize;

/// Number of simultaneous finger contacts a spell requires
/// (`fingers = 2` in a spell TOML).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FingerCount(pub usize);

impl FingerCount {
    /// Creates a finger count.
    pub const fn new(count: usize) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for FingerCount {
    fn from(count: usize) -> Self {
        Self::new(count)
    }
}
