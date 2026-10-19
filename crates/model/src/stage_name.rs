use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Name of an event-pipeline stage (EVENT_PIPELINE.md).
///
/// Profiles scope to a stage via `stage = "game"`; the pipeline resolves
/// the name to a `StageId` at registration. `None` scopes to the general
/// stage - the unscoped default holding all untagged patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
