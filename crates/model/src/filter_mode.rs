#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// Suppression policy of the filter proxy (FILTER_PROXY_VIRTUAL.md).
///
/// Controls when a slot classified as a tangible becomes invisible to the
/// operating system and how unclassified new contacts are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
// Under `cynic` the derive owns serde (SCREAMING_SNAKE wire form
// matching the schema); without it, serde serializes the Rust variant
// names (legacy wire form used by recordings and snapshots).
pub enum FilterMode {
    /// Proxy disabled: every event passes through unchanged.
    Off,
    /// Suppress slots on the first matched frame. Zero leak for real
    /// tangibles, but a one-frame phantom match (fingers coincidentally
    /// forming a profile) eats the contact for the full hysteresis window.
    Immediate,
    /// Suppress only after `STABILITY_GATE` consecutive matched frames -
    /// phantom matches never suppress; a tangible leaks ~1 frame on
    /// placement.
    Gated,
    /// Hold every new contact's events until the first frame evaluation
    /// classifies it: tangible contacts never reach the system at all,
    /// finger contacts appear ~1 frame late but complete. Safest policy.
    Guarded,
    /// Suppress every contact - tangible and finger alike. Tangibles are
    /// still detected internally (engine, interfaces, tracked poses), but
    /// the virtual device emits no touch events at all. For setups where
    /// only figure state matters and touch input must not disturb the UI.
    TangiblesOnly,
}

impl Default for FilterMode {
    /// The default policy balances tangible invisibility against finger
    /// latency.
    fn default() -> Self {
        Self::Guarded
    }
}

impl std::fmt::Display for FilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Off => "off",
            Self::Immediate => "immediate",
            Self::Gated => "gated",
            Self::Guarded => "guarded",
            Self::TangiblesOnly => "tangibles-only",
        };
        write!(f, "{name}")
    }
}

impl std::str::FromStr for FilterMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "off" => Ok(Self::Off),
            "immediate" => Ok(Self::Immediate),
            "gated" => Ok(Self::Gated),
            "guarded" => Ok(Self::Guarded),
            "tangibles-only" | "tangibles_only" => Ok(Self::TangiblesOnly),
            _ => Err(()),
        }
    }
}
