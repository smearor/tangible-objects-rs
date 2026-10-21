use serde::Deserialize;
use serde::Serialize;

/// How likely a same-size contact set of two profiles collides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MatchRisk {
    /// Even the closest subset pairing stays beyond the partial
    /// tolerance - effectively no phantom surface.
    Safe,
    /// Closest pairing fits within partial tolerance (1.5x) but not
    /// exact - phantom possible only through partial spawns.
    Partial,
    /// Closest pairing fits within exact tolerance - phantom possible
    /// even through exact spawns.
    Exact,
}
