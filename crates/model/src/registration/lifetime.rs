use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// How long a registration lives (DYNAMIC_PROFILES.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[serde(rename_all = "kebab-case")]
pub enum Lifetime {
    /// Removed on `unregister*`/`unregister_owner` or when the owning
    /// interface session ends (where disconnect is detectable).
    #[default]
    Session,
    /// Additionally written to disk so it survives daemon restarts.
    Persistent,
}

impl FromStr for Lifetime {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(Self::Session),
            "persistent" => Ok(Self::Persistent),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown lifetime: {s}"),
            )),
        }
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Session => write!(f, "session"),
            Self::Persistent => write!(f, "persistent"),
        }
    }
}
