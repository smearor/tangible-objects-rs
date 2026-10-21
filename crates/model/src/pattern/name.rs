use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Name of a figure profile - the identity used to address a pattern in
/// maps, interfaces and the runtime store (not merely a display string).
///
/// Distinct from [`crate::Owner`]: the owner is *who* registered a
/// profile; the pattern name is *what* the profile is called.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct PatternName(String);

impl PatternName {
    /// Creates a pattern name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the name carries the ` (partial)` match suffix a partial
    /// match produces (`"FM4 (partial)"` -> `"FM4"`).
    pub fn base(&self) -> &str {
        self.0.strip_suffix(" (partial)").unwrap_or(&self.0)
    }
}

impl From<String> for PatternName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for PatternName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for PatternName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for PatternName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for PatternName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(PatternName);
