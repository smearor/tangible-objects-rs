use std::fmt::Display;
use std::str::FromStr;

#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// How long a registration lives (DYNAMIC_PROFILES.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(not(feature = "cynic"), serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
// Under `cynic` the derive owns serde (SCREAMING_SNAKE wire form);
// without it serde keeps the kebab-case form used by registrations.
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
