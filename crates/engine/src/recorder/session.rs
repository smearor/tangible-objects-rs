use std::io::Result;
use std::path::PathBuf;

use tangible_objects_model::Calibration;
use tangible_objects_model::EventCount;
use tangible_objects_model::FrameCount;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::RawEvent;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::RemainingFrameCount;
use tangible_objects_model::TrackedTangible;

use super::FrameContext;
use super::FrameRecorder;
use super::RawRecorder;
use crate::EngineFrame;

/// A running recording session: paired frame + raw sinks with labeling.
///
/// Bundles [`FrameRecorder`] and [`RawRecorder`] writing `<base_path>.jsonl`
/// and `<base_path>.evtest`. The caller supplies session-relative timestamps
/// so the same session type serves live recording (monotonic clock) and
/// deterministic replay recording (evtest timestamps).
///
/// Two label channels exist: a persistent session label (`--record-label`,
/// `SetRecordLabel`) and a temporary mark ([`Self::mark_label`]) that applies
/// to the next N recorded frames - the interactive keypress path.
pub struct RecordingSession {
    /// Base path the artifacts are written to (without extension).
    pub base_path: PathBuf,
    /// JSONL writer for processed frames (`<base>.jsonl`).
    frame_recorder: FrameRecorder,
    /// evtest writer for raw events (`<base>.evtest`).
    raw_recorder: RawRecorder,
    /// Calibration snapshot recorded into every frame's context.
    calibration: Option<Calibration>,
    /// Session-wide ground-truth label (e.g. the figure being recorded).
    label: Option<RecordLabel>,
    /// Temporary label override for `SetRecordLabel` - takes precedence
    /// over `label` while its frame budget lasts.
    label_override: Option<LabelOverride>,
    /// Frames recorded so far - the session's own counter, separate from
    /// the frame numbers which count `SYN_REPORT`s including skipped
    /// empty frames.
    recorded_frames: FrameCount,
}

/// Active mark-label override: applies `label` for `remaining` more
/// recorded frames.
struct LabelOverride {
    /// Temporary ground-truth label.
    label: RecordLabel,
    /// Remaining budget - decremented per recorded frame, saturates at zero
    /// and expires the override.
    remaining: RemainingFrameCount,
}

impl RecordingSession {
    /// Starts a session writing `<base_path>.jsonl` + `<base_path>.evtest`.
    pub fn start(
        base_path: PathBuf,
        device_name: Option<&str>,
        calibration: Option<&Calibration>,
        label: Option<RecordLabel>,
    ) -> Result<Self> {
        let frame_recorder = FrameRecorder::spawn(&base_path.with_extension("jsonl"))?;
        let raw_recorder = RawRecorder::spawn(
            &base_path.with_extension("evtest"),
            device_name,
            calibration,
        )?;
        Ok(Self {
            base_path,
            frame_recorder,
            raw_recorder,
            calibration: calibration.copied(),
            label,
            label_override: None,
            recorded_frames: FrameCount::new(0),
        })
    }

    /// Queues a raw event for the `evtest` artifact.
    ///
    /// `timestamp_secs` is the session-relative event time in seconds.
    pub fn record_event(&self, event: &RawEvent, timestamp_secs: f64) {
        self.raw_recorder.record(event, timestamp_secs);
    }

    /// Records a non-empty frame; returns whether the frame was written.
    ///
    /// Frames without touches and tracked tangibles are skipped per the
    /// recording rules - the caller still counts `frame_no` for every
    /// `SYN_REPORT` so numbering stays aligned.
    pub fn record_frame(
        &mut self,
        frame_no: FrameNumber,
        timestamp_ms: Milliseconds,
        frame: &EngineFrame,
        tracked: &[TrackedTangible],
    ) -> bool {
        if frame.touches.is_empty() && tracked.is_empty() {
            return false;
        }
        let label = self.take_label();
        self.frame_recorder.record(
            frame_no,
            timestamp_ms,
            frame,
            tracked,
            &FrameContext {
                label: label.as_ref(),
                calibration: self.calibration.as_ref(),
                raw_dropped: self.raw_recorder.dropped_count(),
            },
        );
        self.recorded_frames += 1;
        true
    }

    /// The label currently applied to recorded frames (override or session
    /// default).
    pub fn label(&self) -> Option<&RecordLabel> {
        match &self.label_override {
            Some(over) if !over.remaining.is_expired() => Some(&over.label),
            _ => self.label.as_ref(),
        }
    }

    /// Changes the persistent session label (`ControlCommand::SetRecordLabel`
    /// path). A pending [`mark_label`](Self::mark_label) override still wins
    /// for its remaining frames.
    pub fn set_label(&mut self, label: Option<RecordLabel>) {
        self.label = label;
    }

    /// Temporarily applies `label` to the next `frames` recorded frames -
    /// the interactive keypress path; afterwards the session label applies
    /// again.
    pub fn mark_label(&mut self, label: RecordLabel, frames: u64) {
        self.label_override = Some(LabelOverride {
            label,
            remaining: RemainingFrameCount::new(frames),
        });
    }

    /// Frames recorded so far (non-empty frames only).
    pub fn recorded_frames(&self) -> FrameCount {
        self.recorded_frames
    }

    /// Frames dropped because the frame sink's buffer was full.
    pub fn dropped_frames(&self) -> FrameCount {
        self.frame_recorder.dropped_count()
    }

    /// Raw events dropped because the raw sink's buffer was full.
    pub fn dropped_events(&self) -> EventCount {
        self.raw_recorder.dropped_count()
    }

    /// Drains, flushes and closes both artifacts.
    pub fn finish(self) -> Result<()> {
        self.frame_recorder.finish()?;
        self.raw_recorder.finish()
    }

    /// Resolves the effective label for the next recorded frame: the mark
    /// override while budget remains, else the session label.
    fn take_label(&mut self) -> Option<RecordLabel> {
        if let Some(over) = &mut self.label_override {
            if over.remaining.is_expired() {
                self.label_override = None;
            } else {
                let label = over.label.clone();
                over.remaining -= 1;
                return Some(label);
            }
        }
        self.label.clone()
    }
}
