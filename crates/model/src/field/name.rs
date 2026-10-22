use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Name of an occupancy field set - the identity used to address a set
/// in maps, interfaces and the runtime store (not merely a display
/// string).
///
/// Distinct from [`crate::PatternName`]: figure profiles and field sets
/// live in separate namespaces.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct FieldSetName(String);

impl FieldSetName {
    /// Creates a field-set name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FieldSetName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for FieldSetName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl FromStr for FieldSetName {
    type Err = Infallible;

    /// Wraps any string as a field-set name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for FieldSetName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for FieldSetName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for FieldSetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for FieldSetName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(FieldSetName);
