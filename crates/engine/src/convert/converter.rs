use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::DeviceProfile;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RawEvent;

use super::ConvertStats;
use crate::EvtestReplay;
use crate::engine::EngineError;
use crate::recorder::raw::format_event;
use crate::recorder::raw::header_text;

/// Codes carrying positions that convert through the calibration map -
/// both multitouch axes and the legacy single-touch duplicates.
const POSITION_CODES: &[u16] = &[
    53, // ABS_MT_POSITION_X
    54, // ABS_MT_POSITION_Y
    0,  // ABS_X
    1,  // ABS_Y
];

/// Converts a `.evtest` recording of one touchscreen into the raw event
/// stream another device would produce.
///
/// Position values travel raw → mm → raw through both calibrations;
/// timestamps pass through unchanged (the source report cadence is kept -
/// the target's `report_rate_hz` is informational). Events whose
/// `(type, code)` the target does not emit are dropped and counted in
/// [`ConvertStats::dropped`]; out-of-bounds positions clamp to the target
/// axis range ([`ConvertStats::clamped`]).
pub struct DeviceConverter {
    /// Target device - provides the calibration, capability set and the
    /// `evtest` header of the output artifact.
    target: DeviceProfile,
}

impl DeviceConverter {
    /// Creates a converter producing artifacts for `target`.
    pub fn new(target: DeviceProfile) -> Self {
        Self { target }
    }

    /// Converts `input` into `output`; returns conversion statistics.
    pub fn convert(&self, input: &Path, output: &Path) -> Result<ConvertStats, EngineError> {
        let mut replay = EvtestReplay::open(input)?;
        let source = replay.calibration().ok_or_else(|| {
            EngineError::InvalidInput(format!(
                "{} has no calibration header - cannot convert to raw axis values",
                input.display()
            ))
        })?;

        let file = File::create(output)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(
            header_text(Some(&self.target.name), Some(&self.target.calibration())).as_bytes(),
        )?;

        let mut stats = ConvertStats::default();
        while let Some((event, timestamp)) = replay.next_event_timed() {
            stats.events += 1;
            if event.is_syn_report() {
                writer.write_all(format_event(&event, timestamp).as_bytes())?;
                stats.kept += 1;
                continue;
            }
            if !self.target.supports(event.event_type, event.code) {
                stats.dropped += 1;
                continue;
            }
            let converted = self.convert_event(&event, &source, &mut stats);
            writer.write_all(format_event(&converted, timestamp).as_bytes())?;
            stats.kept += 1;
        }
        writer.flush()?;
        Ok(stats)
    }

    /// Maps one event into target raw values. Position codes go through
    /// `raw → mm → raw`; everything else passes through unchanged.
    fn convert_event(
        &self,
        event: &RawEvent,
        source: &tangible_objects_model::Calibration,
        stats: &mut ConvertStats,
    ) -> RawEvent {
        if event.event_type != tangible_objects_model::EventType::Abs
            || !POSITION_CODES.contains(&event.code.as_u16())
        {
            return *event;
        }
        let axis_x = event.code == ABS_MT_POSITION_X
            || event.code == tangible_objects_model::EventCode::new(0);
        let res = if axis_x { source.x_res } else { source.y_res };
        let mm = Millimeters::new(event.value as f64 / res.as_f64());
        let (raw, clamped) = self.target.to_raw(mm, axis_x);
        if clamped {
            stats.clamped += 1;
        }
        RawEvent::new(event.event_type, event.code, raw.as_f64().round() as i32)
    }
}
