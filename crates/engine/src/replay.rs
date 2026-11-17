use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::path::Path;

use tangible_objects_model::Calibration;
use tangible_objects_model::EventCode;
use tangible_objects_model::RawEvent;

use crate::EngineError;

/// Streaming parser for `evtest` text output.
///
/// Reads events one by one from an `evtest` recording and extracts the device
/// calibration (resolution + bounds of the position axes) from the header
/// block before the `Testing ...` line.
pub struct EvtestReplay {
    /// Buffered reader over the recording file - events stream out one
    /// per `next_event` call.
    reader: BufReader<File>,
    /// Calibration extracted from the evtest header (`ABS_MT_POSITION_X/Y`
    /// resolution + bounds); `None` when the header carried no axes.
    calibration: Option<Calibration>,
    /// Device name from the `Input device name:` header line, if any.
    device_name: Option<String>,
    /// Device vendor from the `Input device ID:` header line, if any.
    vendor: Option<u16>,
    /// Device product from the `Input device ID:` header line, if any.
    product: Option<u16>,
}

impl EvtestReplay {
    /// Opens an `evtest` recording and parses its calibration header.
    pub fn open(path: &Path) -> Result<Self, EngineError> {
        let file = File::open(path)?;
        let mut replay = Self {
            reader: BufReader::new(file),
            calibration: None,
            device_name: None,
            vendor: None,
            product: None,
        };
        replay.parse_header();
        Ok(replay)
    }

    /// Parses the evtest header to extract resolution and min/max bounds of
    /// the `ABS_MT_POSITION_X/Y` axes.
    fn parse_header(&mut self) {
        let mut current_code: Option<EventCode> = None;
        let mut x_res: Option<f64> = None;
        let mut y_res: Option<f64> = None;
        let mut x_min: Option<f64> = None;
        let mut x_max: Option<f64> = None;
        let mut y_min: Option<f64> = None;
        let mut y_max: Option<f64> = None;

        let _ = self.reader.seek(std::io::SeekFrom::Start(0));

        let mut line = String::new();
        loop {
            line.clear();
            let bytes = match self.reader.read_line(&mut line) {
                Ok(bytes) => bytes,
                Err(_) => break,
            };
            if bytes == 0 {
                break;
            }

            if line.contains("Testing ...") {
                break;
            }

            if let Some(name) = line
                .strip_prefix("Input device name: \"")
                .and_then(|rest| rest.strip_suffix("\"\n"))
            {
                self.device_name = Some(name.to_string());
            }

            // `Input device ID: bus 0x3 vendor 0x457 product 0x11dc ...`
            if line.starts_with("Input device ID:") {
                self.vendor = parse_hex_field(&line, "vendor");
                self.product = parse_hex_field(&line, "product");
            }

            if line.contains("Event code") && line.contains("ABS_MT_POSITION_X") {
                current_code = Some(tangible_objects_model::ABS_MT_POSITION_X);
            } else if line.contains("Event code") && line.contains("ABS_MT_POSITION_Y") {
                current_code = Some(tangible_objects_model::ABS_MT_POSITION_Y);
            } else if line.contains("Event code") {
                current_code = None;
            }

            if let Some(code) = current_code {
                if let Some(res) = parse_value_line(&line, "Resolution") {
                    match code {
                        tangible_objects_model::ABS_MT_POSITION_X => x_res = Some(res),
                        tangible_objects_model::ABS_MT_POSITION_Y => y_res = Some(res),
                        _ => {}
                    }
                }
                if let Some(min) = parse_value_line(&line, "Min") {
                    match code {
                        tangible_objects_model::ABS_MT_POSITION_X => x_min = Some(min),
                        tangible_objects_model::ABS_MT_POSITION_Y => y_min = Some(min),
                        _ => {}
                    }
                }
                if let Some(max) = parse_value_line(&line, "Max") {
                    match code {
                        tangible_objects_model::ABS_MT_POSITION_X => x_max = Some(max),
                        tangible_objects_model::ABS_MT_POSITION_Y => y_max = Some(max),
                        _ => {}
                    }
                }
            }
        }

        let _ = self.reader.seek(std::io::SeekFrom::Start(0));

        if let (Some(x), Some(y)) = (x_res, y_res)
            && x > 0.0
            && y > 0.0
        {
            self.calibration = Some(Calibration::with_bounds(
                x,
                y,
                x_min.unwrap_or(0.0),
                x_max.unwrap_or(0.0),
                y_min.unwrap_or(0.0),
                y_max.unwrap_or(0.0),
            ));
        }
    }

    /// Device calibration extracted from the header, if present.
    pub fn calibration(&self) -> Option<Calibration> {
        self.calibration
    }

    /// Device name from the `Input device name:` header line, if present.
    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    /// Device `vendor:product` pair from the `Input device ID:` header
    /// line - resolves the recording against the device registry the
    /// same way the daemon resolves a live device.
    pub fn device_ids(&self) -> Option<(u16, u16)> {
        self.vendor.zip(self.product)
    }

    /// Returns the next parsed event, skipping non-event lines.
    /// Returns `None` at EOF.
    pub fn next_event(&mut self) -> Option<RawEvent> {
        self.next_event_timed().map(|(event, _)| event)
    }

    /// Returns the next parsed event with its `evtest` timestamp in seconds.
    ///
    /// Recording passes these timestamps through so replayed sessions produce
    /// deterministic artifacts. Returns `None` at EOF.
    pub fn next_event_timed(&mut self) -> Option<(RawEvent, f64)> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self.reader.read_line(&mut line).ok()?;
            if bytes == 0 {
                return None;
            }
            if let Some(timed) = parse_evtest_line_timed(&line) {
                return Some(timed);
            }
        }
    }
}

/// Parses a single evtest event line into a [`RawEvent`].
fn parse_evtest_line(line: &str) -> Option<RawEvent> {
    if !line.contains("Event:") {
        return None;
    }

    // Delimiter form only: "Event: time ..., ----- SYN_REPORT -----".
    // A typed line like `code 0 (SYN_REPORT)` falls through to the typed
    // parse and still yields a SYN event - while `code 0 (ABS_X)` lines
    // (legacy single-touch axes) no longer misread as frame boundaries.
    if line.contains("SYN_REPORT") && !line.contains("type ") {
        return Some(RawEvent::syn_report());
    }

    // Format: "Event: time X.XXXXXX, type N (EV_XXX), code M (XXX), value V"
    let event_type = parse_u16_after(line, "type ")?;
    let code = parse_u16_after(line, "code ")?;
    let value = parse_i32_after(line, "value ")?;

    Some(RawEvent::new(event_type, code, value))
}

/// Parses a single evtest event line into a [`RawEvent`] plus its timestamp
/// in seconds (`time S.UUUUUU` field).
fn parse_evtest_line_timed(line: &str) -> Option<(RawEvent, f64)> {
    let event = parse_evtest_line(line)?;
    let start = line.find("time ")? + "time ".len();
    let rest = &line[start..];
    let end = rest.find([',', ' '])?;
    let timestamp = rest[..end].parse::<f64>().ok()?;
    Some((event, timestamp))
}

fn parse_u16_after(line: &str, prefix: &str) -> Option<u16> {
    let start = line.find(prefix)? + prefix.len();
    let rest = &line[start..];
    let end = rest.find(' ')?;
    rest[..end].parse::<u16>().ok()
}

fn parse_i32_after(line: &str, prefix: &str) -> Option<i32> {
    let start = line.find(prefix)? + prefix.len();
    let rest = line[start..].trim_end();
    rest.parse::<i32>().ok()
}

fn parse_value_line(line: &str, keyword: &str) -> Option<f64> {
    if !line.contains(keyword) {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts.last().and_then(|s| s.parse::<f64>().ok())
    } else {
        None
    }
}

/// Hex field of an `Input device ID:` line - `vendor 0x457` /
/// `product 0x11dc` style.
fn parse_hex_field(line: &str, field: &str) -> Option<u16> {
    line.split_whitespace()
        .skip_while(|part| *part != field)
        .nth(1)
        .and_then(|value| value.strip_prefix("0x"))
        .and_then(|hex| u16::from_str_radix(hex, 16).ok())
}

#[cfg(test)]
mod tests {
    use super::parse_evtest_line;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::EventCode;
    use tangible_objects_model::EventType;

    #[test]
    fn parses_abs_event_line() {
        let line = "Event: time 1696584000.123456, type 3 (EV_ABS), code 53 (ABS_MT_POSITION_X), value 1234";
        let event = parse_evtest_line(line).unwrap();
        assert_eq!(event.event_type, EventType::Abs);
        assert_eq!(event.code, ABS_MT_POSITION_X);
        assert_eq!(event.value, 1234);
    }

    #[test]
    fn code_zero_abs_event_is_not_a_syn_report() {
        // Regression: the SiS controller emits legacy ABS_X (code 0) axes;
        // a wrongly named `(SYN_REPORT)` in such a line must not split frames.
        let line =
            "Event: time 1696584000.123456, type 3 (EV_ABS), code 0 (SYN_REPORT), value 2401";
        let event = parse_evtest_line(line).unwrap();
        assert_eq!(event.event_type, EventType::Abs);
        assert_eq!(event.code, EventCode::new(0));
        assert!(!event.is_syn_report());
    }

    #[test]
    fn typed_syn_report_line_is_still_a_syn_report() {
        // Some evtest variants write SYN_REPORT as a regular typed event.
        let line = "Event: time 1696584000.123456, type 0 (EV_SYN), code 0 (SYN_REPORT), value 0";
        let event = parse_evtest_line(line).unwrap();
        assert!(event.is_syn_report());
    }

    #[test]
    fn parses_syn_report_line() {
        let line = "Event: time 1696584000.123456, -------------- SYN_REPORT ------------";
        let event = parse_evtest_line(line).unwrap();
        assert!(event.is_syn_report());
    }

    #[test]
    fn ignores_non_event_lines() {
        assert!(parse_evtest_line("Input device name: \"Touchscreen\"").is_none());
        assert!(parse_evtest_line("").is_none());
    }

    /// Regression: replaying the recorded fixtures must produce the same
    /// frame and detection counts as the reference implementation.
    #[test]
    fn fixture_replays_produce_stable_detections() {
        use std::path::PathBuf;

        use super::EvtestReplay;
        use crate::Engine;
        use tangible_objects_model::ProfileFile;

        use tangible_objects_model::DeviceRegistry;

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        // Profiles whose figures no longer exist physically live under
        // `tests/fixtures/profiles` - merged here so fixture replays keep
        // their detection coverage without polluting the production
        // `profiles/` directory.
        let mut dir = ProfileFile::load_dir(&root.join("profiles"));
        dir.profiles
            .extend(ProfileFile::load_dir(&root.join("tests/fixtures/profiles")).profiles);
        let patterns = dir.into_patterns();
        assert!(!patterns.is_empty(), "regression profiles missing");
        let registry = DeviceRegistry::builtin();

        // (fixture, expected total frames, expected frames with tracked tangibles)
        let cases: [(&str, u64, u64); 5] = [
            ("figure-tracking.txt", 0, 0),
            ("figure-tracking-2.txt", 0, 0),
            ("figure-tracking-3.txt", 692, 0),
            ("figure-tracking-4.txt", 595, 566),
            ("figure-tracking-5.txt", 572, 524),
        ];
        for (name, want_frames, want_detected) in cases {
            let mut replay = EvtestReplay::open(&root.join("tests").join("fixtures").join(name))
                .unwrap_or_else(|e| panic!("open {name}: {e}"));
            let mut engine = Engine::new(patterns.clone());
            // The device profile's measured resolution wins over the
            // firmware-reported header values - same resolution the
            // daemon applies to the live device.
            let calibration = replay
                .device_ids()
                .and_then(|(vendor, product)| registry.find_by_ids(vendor, product))
                .map(|profile| profile.calibration())
                .or_else(|| replay.calibration());
            if let Some(calibration) = calibration {
                engine.set_calibration(calibration);
            }
            let mut frames = 0u64;
            let mut detected = 0u64;
            while let Some(event) = replay.next_event() {
                if engine.process(&event).is_some() {
                    frames += 1;
                    if !engine.tracked().is_empty() {
                        detected += 1;
                    }
                }
            }
            assert_eq!((frames, detected), (want_frames, want_detected), "{name}");
        }
    }
}
