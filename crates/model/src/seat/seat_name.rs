use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Application-facing name of a [`crate::TableTopSeat`].
///
/// Unique within the seat's session - the application chooses it
/// (e.g. "player-1"); it is a label for interfaces and the GUI's
/// Seats card, not a user identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct SeatName(String);

impl SeatName {
    /// Creates a seat name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SeatName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for SeatName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl FromStr for SeatName {
    type Err = Infallible;

    /// Wraps any string as a seat name (CLI argument form).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for SeatName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for SeatName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for SeatName {
    const SIGNATURE: &'static zvariant::Signature = <String as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SeatName);
