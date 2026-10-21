use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Application that registered a profile or field set (DYNAMIC_PROFILES.md).
///
/// Owners drive cleanup: `unregister_owner` removes every registration of
/// an owner, and D-Bus binds owners to the caller's unique bus name so a
/// client exit automatically drops its registrations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct Owner(String);

impl Owner {
    /// Creates an owner from an application/connection name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The owner's name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Owner {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for Owner {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for Owner {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for Owner {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(Owner);
