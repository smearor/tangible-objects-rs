use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use super::FrameNumber;
use crate::Calibration;
use crate::CandidateReport;
use crate::EventCount;
use crate::Milliseconds;
use crate::RecordLabel;
use crate::TouchPoint;
use crate::TrackedRecord;

/// Schema version of the JSONL recording format.
///
/// Written into every [`FrameRecord`] as `"v"`; analysis tooling must refuse
/// unknown versions. Bump when the record shape changes.
pub const RECORD_SCHEMA_VERSION: u16 = 1;

/// One JSONL line of a frame recording (TRAINING.md schema `"v": 1`).
///
/// Written once per non-empty `SYN_REPORT` frame — never for frames without
/// touches and tracked tangibles. All spatial quantities are in
/// [`Millimeters`](crate::Millimeters); when `calibration` is `null` the
/// device was uncalibrated and positions are raw axis values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameRecord {
    /// Schema version; serialized as `"v"` and always
    /// [`RECORD_SCHEMA_VERSION`] for newly written records.
    #[serde(rename = "v")]
    pub schema_version: u16,
    /// Sequential frame number within the session.
    pub frame: FrameNumber,
    /// Milliseconds since recording start.
    pub timestamp_ms: Milliseconds,
    /// Ground-truth label of the frame; `null` for unlabeled frames.
    pub label: Option<RecordLabel>,
    /// Calibration used for the raw→mm conversion; `null` when the device
    /// was uncalibrated.
    pub calibration: Option<Calibration>,
    /// Active touch points in this frame.
    pub touches: Vec<TouchPoint>,
    /// All tested candidate combinations — matched and rejected.
    pub candidates: Vec<CandidateReport>,
    /// Tracker state after processing this frame.
    pub tracked: Vec<TrackedRecord>,
    /// Running count of raw events dropped because the raw sink's write
    /// buffer was full. Non-zero breaks exact replay round-trip equality.
    pub raw_dropped: EventCount,
}

impl FrameRecord {
    /// The best candidate per pattern in this frame — the candidate whose
    /// [`required_tolerance`](CandidateReport::required_tolerance) is lowest.
    ///
    /// Used by the collision analysis: for each pattern this is the closest
    /// it came to matching the frame's touch geometry.
    pub fn best_candidates(&self) -> BTreeMap<&str, &CandidateReport> {
        let mut best: BTreeMap<&str, &CandidateReport> = BTreeMap::new();
        for candidate in &self.candidates {
            let replace = match best.get(candidate.pattern_name.as_str()) {
                Some(current) => candidate.required_tolerance() < current.required_tolerance(),
                None => true,
            };
            if replace {
                best.insert(&candidate.pattern_name, candidate);
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::FrameRecord;
    use super::RECORD_SCHEMA_VERSION;
    use crate::EventCount;
    use crate::Millimeters;
    use crate::Milliseconds;
    use crate::PatternId;
    use crate::PatternTolerance;
    use crate::TouchPoint;
    use crate::TouchPointSlot;
    use crate::TrackingId;
    use crate::Vec2;
    use crate::recording::CandidateReport;

    fn sample_record() -> FrameRecord {
        FrameRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            frame: 1024.into(),
            timestamp_ms: Milliseconds::new(12345678),
            label: None,
            calibration: None,
            touches: vec![TouchPoint {
                slot: TouchPointSlot::new(0),
                tracking_id: TrackingId::new(240),
                pos: Vec2::new(192.78, 94.27),
            }],
            candidates: vec![CandidateReport {
                pattern_id: PatternId::new(5),
                pattern_name: "TestFigur-5P".to_string(),
                touch_indices: vec![0, 1, 2, 3],
                matched: true,
                partial: false,
                measured_distances: vec![Millimeters::new(34.15)],
                deviations: vec![Millimeters::new(0.83)],
                tolerance: PatternTolerance::new(3.0),
                tolerances: Vec::new(),
                score: None,
                match_error: Millimeters::new(5.69),
            }],
            tracked: vec![],
            raw_dropped: EventCount::default(),
        }
    }

    #[test]
    fn frame_record_serializes_schema_v1_shape() {
        let json = serde_json::to_value(sample_record()).expect("serialize");

        assert_eq!(json["v"], 1);
        assert_eq!(json["frame"], 1024);
        assert_eq!(json["timestamp_ms"], 12345678);
        assert_eq!(json["raw_dropped"], 0);
        assert_eq!(json["touches"][0]["slot"], 0);
        assert_eq!(json["touches"][0]["tracking_id"], 240);
        assert_eq!(json["touches"][0]["x"], 192.78);
        assert_eq!(json["candidates"][0]["pattern_id"], 5);
        assert_eq!(json["candidates"][0]["tolerance"], 3.0);
        assert_eq!(json["candidates"][0]["match_error"], 5.69);
    }

    #[test]
    fn absent_label_and_calibration_serialize_as_null_markers() {
        let json = serde_json::to_value(sample_record()).expect("serialize");
        assert!(json["label"].is_null());
        assert!(json["calibration"].is_null());
    }
}
