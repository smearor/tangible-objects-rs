use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Ground-truth label attached to recorded frames.
///
/// Set via `--record-label`, an interactive keypress or the interface APIs;
/// identifies which figure (if any) was actually on the surface while the
/// frame was captured. Analysis treats labeled frames as positive samples
/// for the labeled profile and all other candidates as negatives.
/// Serializes as a bare string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordLabel(pub String);

impl RecordLabel {
    /// Creates a record label from a name or pattern identifier.
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    /// The label text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for RecordLabel {
    fn from(label: String) -> Self {
        Self(label)
    }
}

impl From<&str> for RecordLabel {
    fn from(label: &str) -> Self {
        Self(label.to_string())
    }
}

impl Display for RecordLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
