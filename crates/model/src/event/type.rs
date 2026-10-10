/// Linux input event type.
///
/// Only the variants used by the pipeline have named representations; all
/// other numeric values map to [`EventType::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Synchronization event (`EV_SYN`), delimits one input frame.
    Syn,
    /// Key/button event (`EV_KEY`).
    Key,
    /// Absolute axis event (`EV_ABS`) — multitouch slots, positions and
    /// tracking IDs.
    Abs,
    /// Any other event type not interpreted by the pipeline.
    Other(u16),
}

impl From<u16> for EventType {
    fn from(value: u16) -> Self {
        match value {
            0x00 => Self::Syn,
            0x01 => Self::Key,
            0x03 => Self::Abs,
            other => Self::Other(other),
        }
    }
}

impl From<EventType> for u16 {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::Syn => 0x00,
            EventType::Key => 0x01,
            EventType::Abs => 0x03,
            EventType::Other(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventType;

    #[test]
    fn known_types_round_trip() {
        for (raw, expected) in [
            (0x00, EventType::Syn),
            (0x01, EventType::Key),
            (0x03, EventType::Abs),
        ] {
            let event_type = EventType::from(raw);
            assert_eq!(event_type, expected);
            assert_eq!(u16::from(event_type), raw);
        }
    }

    #[test]
    fn unknown_type_becomes_other() {
        assert_eq!(EventType::from(0x11), EventType::Other(0x11));
        assert_eq!(u16::from(EventType::Other(0x11)), 0x11);
    }
}
