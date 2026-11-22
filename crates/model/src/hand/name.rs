//! `HandName` - identity of a registered handprint
//! (`Unregister`, runtime store, `five-finger-tap` arming
//! bindings).

use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::str::FromStr;

/// Human-readable handprint name - the lookup key like
/// `SpellName` / `PatternName`; typically names the person whose
/// hand the profile records (per-user spells, SPELLS.md).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct HandName(pub String);

impl HandName {
    /// Creates a hand name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HandName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for HandName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for HandName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl FromStr for HandName {
    type Err = Infallible;

    /// Wraps any string as a hand name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for HandName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(HandName);
