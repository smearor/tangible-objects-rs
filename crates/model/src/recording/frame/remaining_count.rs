use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::SubAssign;

/// Remaining budget of a temporary frame-scoped marker — a countdown that
/// depletes as frames are consumed.
///
/// Deliberately distinct from [`super::FrameCount`]: a `FrameCount` only
/// grows (it accumulates), a `RemainingFrameCount` only shrinks towards
/// expiry. Saturating subtraction makes expiry idempotent instead of
/// panicking on underflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemainingFrameCount(pub u64);

impl RemainingFrameCount {
    /// Creates a remaining-frame budget.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The raw remaining count.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Whether the budget is fully consumed.
    pub const fn is_expired(self) -> bool {
        self.0 == 0
    }
}

impl SubAssign<u64> for RemainingFrameCount {
    /// Saturating: the budget clamps at zero instead of underflowing.
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

impl From<u64> for RemainingFrameCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl Display for RemainingFrameCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::RemainingFrameCount;

    #[test]
    fn decrements_to_zero_and_expires() {
        let mut remaining = RemainingFrameCount::new(2);
        assert!(!remaining.is_expired());
        remaining -= 1;
        assert_eq!(remaining.as_u64(), 1);
        remaining -= 1;
        assert!(remaining.is_expired());
    }

    #[test]
    fn saturates_instead_of_underflowing() {
        let mut remaining = RemainingFrameCount::new(1);
        remaining -= 5;
        assert_eq!(remaining.as_u64(), 0);
        assert!(remaining.is_expired());
    }
}
