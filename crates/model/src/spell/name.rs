//! `SpellName` - identity of a registered spell (`Unregister`,
//! runtime store, lifecycle events).

use serde::Deserialize;
use serde::Serialize;

/// Human-readable spell name - the lookup key like `PatternName` /
/// `DeadZoneName`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpellName(pub String);

impl SpellName {
    /// Creates a spell name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SpellName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}
