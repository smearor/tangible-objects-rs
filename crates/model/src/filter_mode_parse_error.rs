/// Error returned when [`FilterMode::from_str`] cannot parse the
/// given mode name.
///
/// Carries the rejected input verbatim - valid values are `off`,
/// `immediate`, `gated`, `guarded`, `tangibles-only`/`tangibles_only`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown filter mode: {0}")]
pub struct FilterModeParseError(pub String);

impl FilterModeParseError {
    /// Creates the error from the rejected input.
    pub(crate) fn new(input: &str) -> Self {
        Self(input.to_string())
    }
}
