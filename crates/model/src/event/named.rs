use std::fmt::Display;

use super::ABS_MT_POSITION_X;
use super::ABS_MT_POSITION_Y;
use super::ABS_MT_SLOT;
use super::ABS_MT_TRACKING_ID;
use super::EventCode;
use super::EventType;
use super::RawEvent;
use super::SYN_REPORT;

/// An event code paired with its event type.
///
/// Linux input code namespaces are per event type — `code 0` means
/// `SYN_REPORT` under `EV_SYN` but `ABS_X` under `EV_ABS` — so a code alone
/// cannot be named. This pair is the smallest unit carrying enough context
/// for a correct `evtest`-style name lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamedEventCode {
    /// The event type the code belongs to.
    pub event_type: EventType,
    /// The code within the type's namespace.
    pub code: EventCode,
}

impl NamedEventCode {
    /// Pairs a code with its event type.
    pub const fn new(event_type: EventType, code: EventCode) -> Self {
        Self { event_type, code }
    }

    /// The `evtest`-style name of the code within its type namespace.
    ///
    /// Unknown combinations yield `"?"` — recording stays lossless because
    /// only the label is unknown, never the numeric pair.
    pub fn name(&self) -> &'static str {
        match self.event_type {
            EventType::Syn => match self.code {
                SYN_REPORT => "SYN_REPORT",
                _ => "?",
            },
            EventType::Key => match self.code.as_u16() {
                330 => "BTN_TOUCH",
                _ => "?",
            },
            EventType::Abs => match self.code {
                ABS_MT_SLOT => "ABS_MT_SLOT",
                ABS_MT_POSITION_X => "ABS_MT_POSITION_X",
                ABS_MT_POSITION_Y => "ABS_MT_POSITION_Y",
                ABS_MT_TRACKING_ID => "ABS_MT_TRACKING_ID",
                EventCode(0) => "ABS_X",
                EventCode(1) => "ABS_Y",
                _ => "?",
            },
            EventType::Other(_) => "?",
        }
    }
}

impl From<&RawEvent> for NamedEventCode {
    fn from(event: &RawEvent) -> Self {
        Self::new(event.event_type, event.code)
    }
}

impl From<RawEvent> for NamedEventCode {
    fn from(event: RawEvent) -> Self {
        Self::new(event.event_type, event.code)
    }
}

impl Display for NamedEventCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::EventCode;
    use super::EventType;
    use super::NamedEventCode;
    use crate::ABS_MT_POSITION_X;
    use crate::ABS_MT_POSITION_Y;

    #[test]
    fn code_zero_is_named_per_type() {
        // The recording bug: code 0 under EV_ABS is ABS_X, not SYN_REPORT.
        assert_eq!(
            NamedEventCode::new(EventType::Abs, EventCode::new(0)).name(),
            "ABS_X"
        );
        assert_eq!(
            NamedEventCode::new(EventType::Syn, EventCode::new(0)).name(),
            "SYN_REPORT"
        );
        assert_eq!(
            NamedEventCode::new(EventType::Abs, EventCode::new(1)).name(),
            "ABS_Y"
        );
    }

    #[test]
    fn known_multitouch_codes_are_named() {
        assert_eq!(
            NamedEventCode::new(EventType::Abs, ABS_MT_POSITION_X).name(),
            "ABS_MT_POSITION_X"
        );
        assert_eq!(
            NamedEventCode::new(EventType::Abs, ABS_MT_POSITION_Y).name(),
            "ABS_MT_POSITION_Y"
        );
        assert_eq!(
            NamedEventCode::new(EventType::Key, EventCode::new(330)).name(),
            "BTN_TOUCH"
        );
    }

    #[test]
    fn unknown_combinations_stay_unnamed() {
        assert_eq!(
            NamedEventCode::new(EventType::Abs, EventCode::new(99)).name(),
            "?"
        );
        assert_eq!(
            NamedEventCode::new(EventType::Other(0x04), EventCode::new(0)).name(),
            "?"
        );
    }
}
