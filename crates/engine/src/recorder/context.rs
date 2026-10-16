use tangible_objects_model::Calibration;
use tangible_objects_model::EventCount;
use tangible_objects_model::RecordLabel;

/// Per-frame recording context passed to [`super::FrameRecorder::record`].
///
/// Groups the values owned by the daemon's device loop: the current
/// ground-truth label, the device calibration snapshot and the running count
/// of raw events dropped by the raw sink.
pub struct FrameContext<'a> {
    /// Ground-truth label written into the frame record.
    pub label: Option<&'a RecordLabel>,
    /// Device calibration snapshot; `None` records the `calibration: null`
    /// marker (positions are then raw axis values).
    pub calibration: Option<&'a Calibration>,
    /// Running count of raw events dropped because the raw sink's buffer was
    /// full.
    pub raw_dropped: EventCount,
}
