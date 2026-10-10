use std::fmt::Display;

/// Linux input event code within an [`crate::EventType`].
///
/// Strongly typed wrapper so event codes cannot be mixed up with event
/// types or raw values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventCode(pub u16);

impl EventCode {
    /// Creates an event code from its numeric value.
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// The raw numeric code.
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

impl From<u16> for EventCode {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<EventCode> for u16 {
    fn from(code: EventCode) -> Self {
        code.0
    }
}

impl Display for EventCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
