use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::DeviceInfo;
use crate::RecordingStatus;

/// Point-in-time snapshot of the detection state.
///
/// Distributed via a `watch` channel: interfaces read the latest snapshot
/// for query-style requests (`list_tangibles`) without replaying the event
/// stream.
#[derive(Debug, Clone, Default)]
pub struct EngineSnapshot {
    /// Currently tracked tangibles (EMA-smoothed poses).
    pub tracked: Vec<TrackedTangible>,
    /// Active touch points of the last processed frame.
    pub touches: Vec<TouchPoint>,
    /// IDs of the currently active Tangible Object Sets.
    pub active_sets: Vec<TangibleSetId>,
    /// Attached input device - set once at session start.
    pub device: Option<DeviceInfo>,
    /// Recording session status, updated every frame while active.
    pub recording: Option<RecordingStatus>,
    /// Active missing-pin completion configuration - read-only view for
    /// interfaces (`set_completion` commands go through the device loop).
    pub completion: Option<tangible_objects_model::CompletionConfig>,
}
