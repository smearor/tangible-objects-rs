use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Tolerance widening factor for incumbent re-matches
/// (TRACKING_STABILIZATION.md): a tracked tangible's slot-bound
/// re-match accepts `match_score <= factor`, so motion distortion must
/// not lose an identity that its slot binding already proves. `1.0`
/// disables widening.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AffinityFactor(f64);

impl AffinityFactor {
    /// Creates a factor.
    pub fn new(factor: f64) -> Self {
        Self(factor)
    }

    /// The neutral factor `1.0` - the multiplicative identity: no
    /// widening at all. The unscaled baseline of partial matching
    /// (spawn-time fingerprints have no slot-binding evidence and
    /// must not inherit the incumbent relaxation).
    pub const fn neutral() -> Self {
        Self(1.0)
    }

    /// The widening factor (`match_score <= factor`).
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl Default for AffinityFactor {
    fn default() -> Self {
        Self(2.0)
    }
}

impl From<f64> for AffinityFactor {
    fn from(factor: f64) -> Self {
        Self::new(factor)
    }
}

impl Display for AffinityFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for AffinityFactor {
    const SIGNATURE: &'static zvariant::Signature = <f64 as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(AffinityFactor);
