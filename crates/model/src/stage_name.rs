use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Name of an event-pipeline stage.
///
/// Profiles scope to a stage via `stage = "game"`; the name keys the
/// stage's scope directly. The name is unique per pipeline). `None`
/// scopes to the general stage - the unscoped default holding all
/// untagged patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct StageName(String);

impl StageName {
    /// Creates a stage name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The stage's name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StageName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for StageName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for StageName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for StageName {
    type Err = Infallible;

    /// Wraps any string as a stage name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl Display for StageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(StageName);

#[cfg(feature = "zbus")]
impl zvariant::Type for StageName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}
