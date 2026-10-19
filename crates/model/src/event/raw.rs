use super::EventCode;
use super::EventType;
use super::SYN_REPORT;

/// Device-agnostic raw input event as `(type, code, value)` triple.
///
/// Mirrors the Linux `input_event` layout so it can be produced from evdev
/// events or parsed from `evtest` recordings without depending on either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawEvent {
    /// Linux input event type (e.g. [`EventType::Syn`], [`EventType::Abs`]).
    pub event_type: EventType,
    /// Event code within the type (e.g. [`crate::ABS_MT_POSITION_X`]).
    pub code: EventCode,
    /// Event value (axis value, tracking ID or 0).
    pub value: i32,
}

impl RawEvent {
    /// Creates a new raw event.
    pub fn new(event_type: impl Into<EventType>, code: impl Into<EventCode>, value: i32) -> Self {
        Self {
            event_type: event_type.into(),
            code: code.into(),
            value,
        }
    }

    /// Returns true when this event is a `SYN_REPORT` frame delimiter.
    pub fn is_syn_report(&self) -> bool {
        self.event_type == EventType::Syn && self.code == SYN_REPORT
    }
}

/// Builds a `RawEvent` from an evdev event - the timestamp is not part of
/// the triple. Available with the `evdev` feature.
#[cfg(feature = "evdev")]
impl From<&evdev::InputEvent> for RawEvent {
    fn from(event: &evdev::InputEvent) -> Self {
        Self::new(event.event_type().0, event.code(), event.value())
    }
}

/// Rebuilds an evdev event from the `(type, code, value)` triple.
///
/// The timestamp is assigned at emission - replay/proxy paths add at most
/// one frame of latency, so source timestamps are not carried over.
/// Available with the `evdev` feature.
#[cfg(feature = "evdev")]
impl From<&RawEvent> for evdev::InputEvent {
    fn from(raw: &RawEvent) -> Self {
        Self::new(
            evdev::EventType(u16::from(raw.event_type)),
            raw.code.as_u16(),
            raw.value,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::EventType;
    use super::RawEvent;
    use crate::ABS_MT_POSITION_X;

    #[test]
    fn syn_report_is_detected() {
        assert!(RawEvent::new(EventType::Syn, 0x00, 0).is_syn_report());
    }

    #[test]
    fn abs_event_is_not_syn_report() {
        assert!(!RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, 100).is_syn_report());
    }
}
