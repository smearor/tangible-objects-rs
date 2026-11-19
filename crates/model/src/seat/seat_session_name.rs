use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Name of a [`crate::SeatSession`] - the identity an application
/// claims and addresses on the interfaces (`claimSeatSession`).
///
/// Distinct from [`crate::Owner`]: the session name is *what* the
/// roster is called; the owner is *which application* claimed it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct SeatSessionName(String);

impl SeatSessionName {
    /// Creates a seat session name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SeatSessionName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for SeatSessionName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl FromStr for SeatSessionName {
    type Err = Infallible;

    /// Wraps any string as a session name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for SeatSessionName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for SeatSessionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for SeatSessionName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SeatSessionName);
