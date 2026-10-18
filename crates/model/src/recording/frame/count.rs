use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Add;
use std::ops::AddAssign;

/// A count of frames - a cardinality, not an ordinal.
///
/// Deliberately distinct from [`super::FrameNumber`]: "frame 47" is a
/// sequence number, "47 frames" is a quantity. Analysis results count frames
/// (matched, labeled, dropped); they never mean a frame's position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameCount(pub u64);

impl FrameCount {
    /// Creates a frame count.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The raw count.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u32> for FrameCount {
    fn from(count: u32) -> Self {
        Self(u64::from(count))
    }
}

impl From<u64> for FrameCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl From<usize> for FrameCount {
    fn from(count: usize) -> Self {
        Self(count as u64)
    }
}

impl From<FrameCount> for u64 {
    fn from(count: FrameCount) -> Self {
        count.0
    }
}

impl Add<FrameCount> for FrameCount {
    type Output = FrameCount;

    /// The sum of two counts is a count.
    fn add(self, rhs: FrameCount) -> FrameCount {
        FrameCount(self.0 + rhs.0)
    }
}

impl AddAssign<u64> for FrameCount {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Display for FrameCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for FrameCount {
    const SIGNATURE: &'static zvariant::Signature = <u64 as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(FrameCount);
