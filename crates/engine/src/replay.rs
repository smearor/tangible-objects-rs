use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::path::Path;

use tangible_objects_model::Calibration;
use tangible_objects_model::EventCode;
use tangible_objects_model::EventType;
use tangible_objects_model::RawEvent;
use tangible_objects_model::SYN_REPORT;

use crate::EngineError;

/// Streaming parser for `evtest` text output.
///
/// Reads events one by one from an `evtest` recording and extracts the device
/// calibration (resolution + bounds of the position axes) from the header
/// block before the `Testing ...` line.
pub struct EvtestReplay {
    reader: BufReader<File>,
    calibration: Option<Calibration>,
}

impl EvtestReplay {
    /// Opens an `evtest` recording and parses its calibration header.
    pub fn open(path: &Path) -> Result<Self, EngineError> {
        let file = File::open(path)?;
        let mut replay = Self {
            reader: BufReader::new(file),
            calibration: None,
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

    /// Returns the next parsed event, skipping non-event lines.
    /// Returns `None` at EOF.
    pub fn next_event(&mut self) -> Option<RawEvent> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self.reader.read_line(&mut line).ok()?;
            if bytes == 0 {
                return None;
            }
            if let Some(event) = parse_evtest_line(&line) {
                return Some(event);
            }
        }
    }
}

/// Parses a single evtest event line into a [`RawEvent`].
fn parse_evtest_line(line: &str) -> Option<RawEvent> {
    if !line.contains("Event:") {
        return None;
    }

    if line.contains("SYN_REPORT") {
        return Some(RawEvent::new(EventType::Syn, SYN_REPORT, 0));
    }

    // Format: "Event: time X.XXXXXX, type N (EV_XXX), code M (XXX), value V"
    let event_type = parse_u16_after(line, "type ")?;
    let code = parse_u16_after(line, "code ")?;
    let value = parse_i32_after(line, "value ")?;

    Some(RawEvent::new(event_type, code, value))
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

#[cfg(test)]
mod tests {
    use super::parse_evtest_line;
    use tangible_objects_model::ABS_MT_POSITION_X;
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
}
