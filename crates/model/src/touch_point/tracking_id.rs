use std::fmt::Display;

/// Kernel-assigned identifier of a touch contact's lifetime.
///
/// Reported via `ABS_MT_TRACKING_ID`; the sentinel [`TrackingId::LIFT_OFF`]
/// marks the release of a contact and frees its slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrackingId(pub i32);

impl TrackingId {
    /// Sentinel the kernel reports when a contact is lifted off the surface.
    pub const LIFT_OFF: Self = Self(-1);

    /// Creates a tracking ID from its raw event value.
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// The raw numeric ID.
    pub const fn as_i32(self) -> i32 {
        self.0
    }

    /// Whether this value is the lift-off sentinel.
    pub const fn is_lift_off(self) -> bool {
        self.0 == Self::LIFT_OFF.0
    }
}

impl From<i32> for TrackingId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<TrackingId> for i32 {
    fn from(id: TrackingId) -> Self {
        id.0
    }
}

impl Display for TrackingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
