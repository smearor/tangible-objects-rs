//! `ProxyRecording` - the paired input/output event capture of the
//! filter proxy (`--record` / `StartRecording` on `--filter`).

use std::io;
use std::path::Path;
use std::time::Instant;

use tangible_objects_engine::RawRecorder;
use tangible_objects_model::Calibration;
use tangible_objects_model::EventCount;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RawEvent;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::RecordingStatus;

/// Paired recording of the filter proxy: `<base>.evtest` captures the
/// raw input stream and `<base>.out.evtest` the emitted OS-visible
/// stream after suppression. Both files carry session-relative
/// timestamps in the same clock base, so `out.ts - in.ts` measures the
/// proxy's per-event latency.
pub(crate) struct ProxyRecording {
    /// Raw input events before engine and pipeline.
    in_recorder: RawRecorder,
    /// The batch emitted on the virtual device per frame.
    out_recorder: RawRecorder,
    /// Clock origin shared by both streams.
    session_start: Instant,
    /// Emitted frames so far - batches containing a `SYN_REPORT`.
    frames: FrameCount,
    /// Ground-truth label carried by the interface-facing status.
    label: Option<RecordLabel>,
}

impl ProxyRecording {
    /// Starts the pair under `base` (`<base>.evtest` +
    /// `<base>.out.evtest`).
    pub(crate) fn start(
        base: &Path,
        device_name: Option<&str>,
        calibration: Option<&Calibration>,
        label: Option<RecordLabel>,
    ) -> io::Result<Self> {
        let in_path = base.with_extension("evtest");
        let out_path = base.with_extension("out.evtest");
        let in_recorder = RawRecorder::spawn(&in_path, device_name, calibration)?;
        let out_recorder = RawRecorder::spawn(&out_path, device_name, calibration)?;
        tracing::info!(
            in_path = %in_path.display(),
            out_path = %out_path.display(),
            "proxy recording active"
        );
        Ok(Self {
            in_recorder,
            out_recorder,
            session_start: Instant::now(),
            frames: FrameCount::new(0),
            label,
        })
    }

    /// Queues one raw input event on the `in` side.
    pub(crate) fn record_in(&self, event: &RawEvent) {
        self.in_recorder
            .record(event, self.session_start.elapsed().as_secs_f64());
    }

    /// Queues the emitted batch on the `out` side; a `SYN_REPORT`
    /// inside the batch counts a written frame.
    pub(crate) fn record_out(&mut self, events: impl Iterator<Item = RawEvent>) {
        let timestamp_secs = self.session_start.elapsed().as_secs_f64();
        for raw in events {
            if raw.is_syn_report() {
                self.frames += 1;
            }
            self.out_recorder.record(&raw, timestamp_secs);
        }
    }

    /// Frames written to the out stream so far.
    pub(crate) fn frames(&self) -> FrameCount {
        self.frames
    }

    /// Sets the ground-truth label.
    pub(crate) fn set_label(&mut self, label: Option<RecordLabel>) {
        self.label = label;
    }

    /// Live status for `recording_status` and `EngineSnapshot`.
    ///
    /// `recorded_frames` counts emitted `SYN_REPORT` batches; the raw
    /// `dropped_events` counter aggregates both sides since the in
    /// and out sinks share the same drop semantics.
    pub(crate) fn status(&self) -> RecordingStatus {
        RecordingStatus {
            active: true,
            label: self.label.clone(),
            recorded_frames: self.frames,
            dropped_frames: FrameCount::new(0),
            dropped_events: EventCount::new(
                self.in_recorder.dropped_count().as_u64()
                    + self.out_recorder.dropped_count().as_u64(),
            ),
        }
    }

    /// Finalizes both files (flush + writer join).
    pub(crate) fn finish(self) -> io::Result<()> {
        self.in_recorder.finish()?;
        self.out_recorder.finish()
    }
}
