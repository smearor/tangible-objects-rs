use std::fmt::Display;
use std::str::FromStr;

#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// Which pass [`DeviationReport`](tangible_objects_analysis::DeviationReport)
/// runs over a recording.
///
/// Shared by the MCP `analyze_recording` tool, the GraphQL
/// `analyzeRecording` query and the D-Bus `analyze_recording` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(not(feature = "cynic"), serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
// Under `cynic` the derive owns serde (SCREAMING_SNAKE wire form);
// without it serde keeps the snake_case form used by replay CLIs.
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
