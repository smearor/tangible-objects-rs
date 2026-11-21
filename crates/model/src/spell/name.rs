//! `SpellName` - identity of a registered spell (`Unregister`,
//! runtime store, lifecycle events).

use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::str::FromStr;

/// Human-readable spell name - the lookup key like `PatternName` /
/// `DeadZoneName`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
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

impl std::fmt::Display for SpellName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for SpellName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl FromStr for SpellName {
    type Err = Infallible;

    /// Wraps any string as a spell name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for SpellName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SpellName);
