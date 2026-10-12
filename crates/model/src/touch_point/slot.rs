use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Multitouch slot number (protocol type B).
///
/// The kernel assigns each active contact a slot; the slot index identifies
/// the contact across subsequent `ABS_MT` events until it is released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TouchPointSlot(pub u32);

impl TouchPointSlot {
    /// Creates a slot identifier from its numeric value.
    pub const fn new(slot: u32) -> Self {
        Self(slot)
    }

    /// The raw numeric slot index.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for TouchPointSlot {
    fn from(slot: u32) -> Self {
        Self(slot)
    }
}

impl From<TouchPointSlot> for u32 {
    fn from(slot: TouchPointSlot) -> Self {
        slot.0
    }
}

impl Display for TouchPointSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
