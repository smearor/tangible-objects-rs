use serde::Deserialize;
use serde::Serialize;

use tangible_objects_model::FrameCount;

/// Labeled presence frames binned by the `required_tolerance /
/// tolerance` ratio.
///
/// Ratios `>= 1.0` exceed the configured tolerance; those frames are
/// counted in `over_tolerance_frames`, not here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RatioBands {
    /// Frames whose best candidate needed under 25 % of the configured
    /// tolerance.
    pub under_25: FrameCount,
    /// Frames needing 25-50 % of the tolerance.
    pub between_25_50: FrameCount,
    /// Frames needing 50-75 % of the tolerance.
    pub between_50_75: FrameCount,
    /// Frames needing 75-100 % of the tolerance - closest to the
    /// acceptance boundary without crossing it.
    pub between_75_100: FrameCount,
}

impl RatioBands {
    /// Counts one frame in the band its ratio falls into -
    /// callers must only pass ratios `< 1.0`.
    pub fn record(&mut self, ratio: f64) {
        let band = if ratio < 0.25 {
            &mut self.under_25
        } else if ratio < 0.5 {
            &mut self.between_25_50
        } else if ratio < 0.75 {
            &mut self.between_50_75
        } else {
            &mut self.between_75_100
        };
        *band += 1;
    }
}

impl std::fmt::Display for RatioBands {
    /// Bands print as their labeled quota ranges.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<25%={} | 25-50%={} | 50-75%={} | 75-100%={}",
            self.under_25, self.between_25_50, self.between_50_75, self.between_75_100
        )
    }
}
