use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

/// Stable identifier of a floating-board card - the wire-shape
/// counterpart of the board's internal `CardId`, used to correlate
/// card diagnostics across interfaces.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct CardId(String);

impl CardId {
    /// Creates a card identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CardId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for CardId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl FromStr for CardId {
    type Err = Infallible;

    /// Wraps any string as a card identifier.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl AsRef<str> for CardId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
