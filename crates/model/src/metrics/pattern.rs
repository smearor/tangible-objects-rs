use serde::Deserialize;
use serde::Serialize;

use crate::ContactCount;
use crate::FrameCount;
use crate::Millimeters;
use crate::TangibleCount;

/// Per-pattern recognition counters of a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternMetrics {
    /// Frames in which at least one tangible of this pattern was tracked.
    pub tracked_frames: FrameCount,
    /// Distinct tangible identities this pattern spawned - `> 1` means
    /// expiry/re-add churn (the real re-identification measure).
    pub tangible_count: TangibleCount,
    /// Frames with an exact match for this pattern.
    pub exact_match_frames: FrameCount,
    /// Frames with a partial match for this pattern.
    pub partial_match_frames: FrameCount,
    /// Total pins claimed by missing-pin completion for this pattern.
    pub claimed_pins: ContactCount,
    /// Worst per-distance deviation of a winning exact match (only when
    /// the frame carried diagnostics).
    pub max_match_deviation: Option<Millimeters>,
}
