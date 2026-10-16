use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TrySendError;
use std::sync::mpsc::sync_channel;
use std::thread::JoinHandle;

use tangible_objects_model::FrameNumber;
use tangible_objects_model::FrameRecord;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::RECORD_SCHEMA_VERSION;
use tangible_objects_model::TrackedRecord;
use tangible_objects_model::TrackedTangible;
use tracing::warn;

use super::FrameContext;
use crate::EngineFrame;

/// Bounded channel capacity between the device loop and the writer thread
/// (~8 seconds at 120 fps).
const FRAME_CHANNEL_CAPACITY: usize = 1024;

/// Writes one JSONL [`FrameRecord`] per recorded engine frame.
///
/// Serialization and file writes happen on a dedicated thread behind a
/// bounded channel: [`FrameRecorder::record`] never blocks the device loop;
/// a full buffer drops frames and counts them in
/// [`FrameRecorder::dropped_count`].
pub struct FrameRecorder {
    sender: Option<SyncSender<FrameRecord>>,
    dropped: Arc<AtomicU64>,
    thread: Option<JoinHandle<std::io::Result<()>>>,
}

impl FrameRecorder {
    /// Creates the JSONL file at `path` and spawns the writer thread.
    pub fn spawn(path: &Path) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let (sender, receiver) = sync_channel::<FrameRecord>(FRAME_CHANNEL_CAPACITY);
        let dropped = Arc::new(AtomicU64::new(0));
        let thread = std::thread::spawn(move || {
            let mut writer = BufWriter::new(file);
            while let Ok(record) = receiver.recv() {
                serde_json::to_writer(&mut writer, &record).map_err(std::io::Error::other)?;
                writer.write_all(b"\n")?;
            }
            writer.flush()
        });
        Ok(Self {
            sender: Some(sender),
            dropped,
            thread: Some(thread),
        })
    }

    /// Records one engine frame.
    ///
    /// `frame_no` counts every `SYN_REPORT` of the session — skipped empty
    /// frames leave gaps, keeping the numbering aligned with the raw stream.
    /// `timestamp_ms` is caller-supplied (session clock live, evtest time on
    /// replay) so replayed recordings stay deterministic.
    /// Never blocks: a full channel drops the frame and counts it.
    pub fn record(
        &self,
        frame_no: u64,
        timestamp_ms: Milliseconds,
        frame: &EngineFrame,
        tracked: &[TrackedTangible],
        context: &FrameContext,
    ) {
        let record = FrameRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            frame: FrameNumber::new(frame_no),
            timestamp_ms,
            label: context.label.cloned(),
            calibration: context.calibration.copied(),
            touches: frame.touches.clone(),
            candidates: frame.candidates.clone(),
            tracked: tracked.iter().map(TrackedRecord::from).collect(),
            raw_dropped: context.raw_dropped,
        };
        let Some(sender) = &self.sender else {
            return;
        };
        match sender.try_send(record) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                if dropped == 1 {
                    warn!("frame recorder buffer full — dropping frames");
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                warn!("frame recorder writer closed before stop");
            }
        }
    }

    /// Running count of frames dropped because the writer buffer was full.
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Closes the channel, drains the writer thread and flushes the file.
    pub fn finish(mut self) -> std::io::Result<()> {
        self.shutdown()
    }

    fn shutdown(&mut self) -> std::io::Result<()> {
        self.sender.take();
        match self.thread.take() {
            Some(thread) => match thread.join() {
                Ok(result) => result,
                Err(_) => Err(std::io::Error::other("frame recorder thread panicked")),
            },
            None => Ok(()),
        }
    }
}

impl Drop for FrameRecorder {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::FrameContext;
    use super::FrameRecorder;
    use crate::EngineFrame;
    use tangible_objects_model::CandidateReport;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Milliseconds;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternTolerance;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("robofigure-{name}-{}", std::process::id()))
    }

    #[test]
    fn writes_one_jsonl_record_per_frame() {
        let path = temp_path("frame-recorder.jsonl");
        let frame = EngineFrame {
            matches: Vec::new(),
            touches: vec![TouchPoint {
                slot: TouchPointSlot::new(0),
                tracking_id: TrackingId::new(240),
                pos: Vec2::new(192.78, 94.27),
            }],
            events: Vec::new(),
            candidates: vec![CandidateReport {
                pattern_id: PatternId::new(5),
                pattern_name: "TestFigur-5P".to_string(),
                touch_indices: vec![0],
                matched: true,
                partial: false,
                measured_distances: vec![Millimeters::new(34.15)],
                deviations: vec![Millimeters::new(0.83)],
                tolerance: PatternTolerance::new(3.0),
                match_error: Millimeters::new(5.69),
            }],
        };

        {
            let recorder = FrameRecorder::spawn(&path).expect("spawn recorder");
            let context = FrameContext {
                label: None,
                calibration: None,
                raw_dropped: 0,
            };
            recorder.record(7, Milliseconds::new(0), &frame, &[], &context);
            recorder.record(8, Milliseconds::new(16), &frame, &[], &context);
            recorder.finish().expect("finish recorder");
        }

        let content = std::fs::read_to_string(&path).expect("read recording");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: serde_json::Value = serde_json::from_str(lines[0]).expect("parse line");
        assert_eq!(first["v"], 1);
        assert_eq!(first["frame"], 7);
        assert_eq!(first["touches"][0]["tracking_id"], 240);
        assert_eq!(first["candidates"][0]["deviations"][0], 0.83);
        assert!(first["label"].is_null());
        assert!(first["calibration"].is_null());
        let second: serde_json::Value = serde_json::from_str(lines[1]).expect("parse line");
        assert_eq!(second["frame"], 8);

        std::fs::remove_file(&path).ok();
    }
}
