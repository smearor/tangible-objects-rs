use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

/// Per-slot position smoothing factor (TRACKING_STABILIZATION.md): EMA
/// weight of the current raw position (`pos = alpha*raw +
/// (1-alpha)*prev`). Averages out the differential-latency distortion
/// panels report during motion. `0.0` disables smoothing, `1.0` uses
/// raw positions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmoothingAlpha(f64);

impl SmoothingAlpha {
    /// Creates a smoothing factor.
    pub fn new(alpha: f64) -> Self {
        Self(alpha)
    }

    /// The EMA weight of the raw position.
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl Default for SmoothingAlpha {
    fn default() -> Self {
        Self(0.6)
    }
}

impl From<f64> for SmoothingAlpha {
    fn from(alpha: f64) -> Self {
        Self::new(alpha)
    }
}

impl Display for SmoothingAlpha {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for SmoothingAlpha {
    const SIGNATURE: &'static zvariant::Signature = <f64 as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SmoothingAlpha);
