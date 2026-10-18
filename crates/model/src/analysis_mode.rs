use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// Which pass [`DeviationReport`](tangible_objects_analysis::DeviationReport)
/// runs over a recording.
///
/// Shared by the MCP `analyze_recording` tool, the GraphQL
/// `analyzeRecording` query and the D-Bus `analyze_recording` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub enum AnalysisMode {
    /// Fold the recorded generic candidates - fast, but blind to frames
    /// whose contacts ran through the incumbent track path.
    #[default]
    Candidates,
    /// Re-evaluate every same-size contact subset of each frame against
    /// the pattern fingerprints - covers incumbent-tracked frames.
    Contacts,
}

impl FromStr for AnalysisMode {
    type Err = String;

    /// Parses `"candidates"` or `"contacts"`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "candidates" => Ok(Self::Candidates),
            "contacts" => Ok(Self::Contacts),
            other => Err(format!("unknown analysis mode: {other}")),
        }
    }
}

impl Display for AnalysisMode {
    /// Writes `"candidates"` or `"contacts"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Candidates => "candidates",
            Self::Contacts => "contacts",
        })
    }
}
