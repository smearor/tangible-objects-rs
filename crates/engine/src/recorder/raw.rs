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

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::ABS_MT_POSITION_Y;
use tangible_objects_model::Calibration;
use tangible_objects_model::EventCode;
use tangible_objects_model::EventType;
use tangible_objects_model::NamedEventCode;
use tangible_objects_model::RawEvent;
use tracing::warn;

/// Bounded channel capacity between the device loop and the writer thread.
/// Covers roughly two seconds of a busy frame (~4k events/s).
const RAW_CHANNEL_CAPACITY: usize = 8192;

/// Writes every raw input event to an `evtest`-compatible text file.
///
/// The header carries the device name and the calibration of the position
/// axes (`Value`/`Min`/`Max`/`Resolution` under `ABS_MT_POSITION_X/Y`), so the
/// artifact replays through `EvtestReplay` and reproduces the recorded frame
/// stream deterministically. Writes happen on a dedicated thread behind a
/// bounded channel: [`RawRecorder::record`] never blocks the device loop; a
/// full buffer drops events and counts them in [`RawRecorder::dropped_count`].
pub struct RawRecorder {
    sender: Option<SyncSender<(RawEvent, f64)>>,
    dropped: Arc<AtomicU64>,
    thread: Option<JoinHandle<std::io::Result<()>>>,
}

impl RawRecorder {
    /// Creates the file at `path`, writes the `evtest` header and spawns the
    /// writer thread.
    pub fn spawn(
        path: &Path,
        device_name: Option<&str>,
        calibration: Option<&Calibration>,
    ) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let header = header_text(device_name, calibration);
        let (sender, receiver) = sync_channel::<(RawEvent, f64)>(RAW_CHANNEL_CAPACITY);
        let dropped = Arc::new(AtomicU64::new(0));
        let thread = std::thread::spawn(move || {
            let mut writer = BufWriter::new(file);
            writer.write_all(header.as_bytes())?;
            while let Ok((event, timestamp_secs)) = receiver.recv() {
                writer.write_all(format_event(&event, timestamp_secs).as_bytes())?;
            }
            writer.flush()
        });
        Ok(Self {
            sender: Some(sender),
            dropped,
            thread: Some(thread),
        })
    }

    /// Queues a raw event for writing.
    ///
    /// `timestamp_secs` is the session-relative event time in seconds —
    /// callers on the replay path pass the evtest timestamp so recorded
    /// artifacts stay deterministic. Never blocks: a full channel drops the
    /// event and counts it; a closed channel logs a warning once.
    pub fn record(&self, event: &RawEvent, timestamp_secs: f64) {
        let Some(sender) = &self.sender else {
            return;
        };
        match sender.try_send((*event, timestamp_secs)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                if dropped == 1 {
                    warn!("raw recorder buffer full — dropping events");
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                warn!("raw recorder writer closed before stop");
            }
        }
    }

    /// Running count of events dropped because the writer buffer was full.
    ///
    /// Reported as `raw_dropped` in the frame stream; non-zero breaks exact
    /// replay round-trip equality.
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
                Err(_) => Err(std::io::Error::other("raw recorder thread panicked")),
            },
            None => Ok(()),
        }
    }
}

impl Drop for RawRecorder {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Formats one event as an `evtest` output line with the given
/// session-relative timestamp in seconds.
fn format_event(event: &RawEvent, timestamp_secs: f64) -> String {
    if event.is_syn_report() {
        return format!(
            "Event: time {timestamp_secs:.6}, -------------- SYN_REPORT ------------\n"
        );
    }
    format!(
        "Event: time {timestamp_secs:.6}, type {} ({}), code {} ({}), value {}\n",
        u16::from(event.event_type),
        event_type_name(event.event_type),
        event.code.as_u16(),
        NamedEventCode::from(event),
        event.value,
    )
}

/// Human-readable name of an event type for the `evtest` line.
fn event_type_name(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Syn => "EV_SYN",
        EventType::Key => "EV_KEY",
        EventType::Abs => "EV_ABS",
        EventType::Other(_) => "EV_OTHER",
    }
}

/// Formats one axis block (`Event code`, `Value`, `Min`, `Max`,
/// `Resolution`) in `evtest` header style. Header codes are always `EV_ABS`.
fn axis_block(
    code: EventCode,
    min: tangible_objects_model::RawAxisValue,
    max: tangible_objects_model::RawAxisValue,
    resolution: tangible_objects_model::UnitsPerMm,
) -> String {
    format!(
        "    Event code {} ({})\n      Value        0\n      Min          {}\n      Max          {}\n      Resolution   {}\n",
        code.as_u16(),
        NamedEventCode::new(EventType::Abs, code),
        min,
        max,
        resolution,
    )
}

/// Builds the `evtest` header carrying device name and position-axis
/// calibration (`Value`/`Min`/`Max`/`Resolution` blocks parsed by
/// `EvtestReplay`).
fn header_text(device_name: Option<&str>, calibration: Option<&Calibration>) -> String {
    let mut text = String::from("Input driver version is 1.0.1\n");
    text.push_str(&format!(
        "Input device name: \"{}\"\n",
        device_name.unwrap_or("recorded device")
    ));
    text.push_str("Supported events:\n  Event type 0 (EV_SYN)\n  Event type 1 (EV_KEY)\n  Event type 3 (EV_ABS)\n");
    if let Some(calibration) = calibration {
        text.push_str(&axis_block(
            ABS_MT_POSITION_X,
            calibration.x_min,
            calibration.x_max,
            calibration.x_res,
        ));
        text.push_str(&axis_block(
            ABS_MT_POSITION_Y,
            calibration.y_min,
            calibration.y_max,
            calibration.y_res,
        ));
    }
    text.push_str("Testing ... (interrupt to exit)\n");
    text
}

#[cfg(test)]
mod tests {
    use super::RawRecorder;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::Calibration;
    use tangible_objects_model::EventType;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::SYN_REPORT;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("robofigure-{name}-{}", std::process::id()))
    }

    #[test]
    fn recording_replays_through_evtest_replay() {
        let path = temp_path("raw-recorder.evtest");
        let calibration = Calibration::with_bounds(7.0, 12.0, 0.0, 4095.0, 0.0, 4095.0);
        let events = vec![
            RawEvent::new(EventType::Abs, ABS_MT_SLOT, 0),
            RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, 240),
            RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, 1927),
            // code 0 under EV_ABS is ABS_X — must not decode as SYN_REPORT
            // (the original recorder bug produced phantom frames this way).
            RawEvent::new(
                EventType::Abs,
                tangible_objects_model::EventCode::new(0),
                2401,
            ),
            RawEvent::new(EventType::Syn, SYN_REPORT, 0),
        ];

        {
            let recorder = RawRecorder::spawn(&path, Some("test device"), Some(&calibration))
                .expect("spawn recorder");
            for (index, event) in events.iter().enumerate() {
                recorder.record(event, index as f64);
            }
            recorder.finish().expect("finish recorder");
        }

        let mut replay = crate::EvtestReplay::open(&path).expect("open replay");
        let replayed_calibration = replay.calibration().expect("calibration in header");
        assert_eq!(replayed_calibration.x_res.as_f64(), 7.0);
        assert_eq!(replayed_calibration.y_res.as_f64(), 12.0);
        assert_eq!(replayed_calibration.x_max.as_f64(), 4095.0);

        let mut replayed = Vec::new();
        while let Some(event) = replay.next_event() {
            replayed.push(event);
        }
        assert_eq!(replayed, events);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn recording_without_calibration_still_replays() {
        let path = temp_path("raw-recorder-nocal.evtest");
        {
            let recorder = RawRecorder::spawn(&path, None, None).expect("spawn recorder");
            recorder.record(&RawEvent::new(EventType::Syn, SYN_REPORT, 0), 0.0);
            recorder.finish().expect("finish recorder");
        }
        let mut replay = crate::EvtestReplay::open(&path).expect("open replay");
        assert!(replay.calibration().is_none());
        assert_eq!(
            replay.next_event().map(|e| e.event_type),
            Some(EventType::Syn)
        );
        std::fs::remove_file(&path).ok();
    }
}
