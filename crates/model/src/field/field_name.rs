use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Name of a single occupancy field (a cell) inside a
/// [`crate::FieldSet`] - the identity used in field events and
/// interfaces, namespaced by its owning set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldName(String);

impl FieldName {
    /// Creates a field name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FieldName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for FieldName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for FieldName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for FieldName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for FieldName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(FieldName);
