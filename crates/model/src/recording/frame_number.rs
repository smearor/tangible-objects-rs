use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Sequential frame number within a recording session.
///
/// Counts `SYN_REPORT` frames from the start of the recording. Frames skipped
/// because they were empty leave gaps, keeping the count aligned with the
/// raw event stream and the device's report rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameNumber(pub u64);

impl FrameNumber {
    /// Creates a frame number.
    pub const fn new(frame: u64) -> Self {
        Self(frame)
    }

    /// The raw frame number.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for FrameNumber {
    fn from(frame: u64) -> Self {
        Self(frame)
    }
}

impl From<FrameNumber> for u64 {
    fn from(frame: FrameNumber) -> Self {
        frame.0
    }
}

impl Display for FrameNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
