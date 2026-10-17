use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::str::FromStr;

/// How a completed partial match is treated downstream.
///
/// A partial match whose missing pin(s) were claimed is no longer a plain
/// partial - the mode decides whether it counts as exact for suppression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[serde(rename_all = "kebab-case")]
pub enum CompletionMode {
    /// The claimed pins complete the match for tracking and slot affinity,
    /// but the match stays partial - its slots are never suppressed.
    /// Conservative default: a wrongly claimed finger is bound but not
    /// swallowed by the filter proxy.
    #[default]
    Match,
    /// A completed partial counts as exact - all its slots, including the
    /// claimed ones, join the suppression set.
    Exact,
}

impl FromStr for CompletionMode {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "match" => Ok(Self::Match),
            "exact" => Ok(Self::Exact),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown completion mode: {s}"),
            )),
        }
    }
}

impl Display for CompletionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Match => write!(f, "match"),
            Self::Exact => write!(f, "exact"),
        }
    }
}
