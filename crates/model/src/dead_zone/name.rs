use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Name of a dead zone - the identity used to address a zone in
/// interfaces and the runtime store (not merely a display string).
///
/// Distinct from [`crate::FieldSetName`] and [`crate::PatternName`]:
/// dead zones live in their own namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct DeadZoneName(String);

impl DeadZoneName {
    /// Creates a dead-zone name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DeadZoneName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for DeadZoneName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl FromStr for DeadZoneName {
    type Err = Infallible;

    /// Wraps any string as a dead-zone name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for DeadZoneName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for DeadZoneName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for DeadZoneName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for DeadZoneName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(DeadZoneName);
