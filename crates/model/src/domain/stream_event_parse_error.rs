/// Error returned when [`crate::StreamEvent::from_str`] cannot parse the
/// given kind name.
///
/// Carries the normalized (uppercased) input - the same string the
/// `SCREAMING_SNAKE` wire vocabulary would expect.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown stream event kind: {0}")]
pub struct StreamEventParseError(pub String);

impl StreamEventParseError {
    /// Creates the error from the rejected input (any case).
    pub(crate) fn new(input: &str) -> Self {
        Self(input.to_uppercase())
    }
}
