//! Replay mode: an `evtest` recording through the detection pipeline.

use std::path::Path;
use std::time::Duration;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::EngineEvent;
use tangible_objects_engine::EvtestReplay;
use tangible_objects_engine::RecordingSession;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::PatternSet;
use tangible_objects_model::RecordLabel;
use tracing::info;
use tracing::warn;

/// Replays an `evtest` recording through the detection pipeline.
///
/// With `record_base` a [`RecordingSession`] additionally writes
/// `<base>.jsonl` + `<base>.evtest`; event timestamps come from the input
/// file itself (offset to the first event), so the artifacts are
/// deterministic.
pub fn run_replay(
    path: &Path,
    patterns: PatternSet<'_>,
    record_base: Option<&Path>,
    label: Option<RecordLabel>,
) -> Result<()> {
    let mut replay = EvtestReplay::open(path).into_diagnostic()?;
    let mut engine = Engine::new(patterns.to_vec());

    if let Some(cal) = replay.calibration() {
        info!(
            "calibration: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        warn!("no calibration in evtest header - profiles must be in raw coordinates");
    }

    let mut session = match record_base {
        Some(base) => Some(
            RecordingSession::start(
                base.to_path_buf(),
                replay.device_name(),
                engine.calibration().as_ref(),
                label,
            )
            .into_diagnostic()?,
        ),
        None => None,
    };

    let mut frame_no: u64 = 0;
    let mut detected_frames: u64 = 0;
    let mut first_timestamp: Option<f64> = None;

    while let Some((event, timestamp)) = replay.next_event_timed() {
        let offset = timestamp - *first_timestamp.get_or_insert(timestamp);
        if let Some(session) = &session {
            session.record_event(&event, offset);
        }
        let engine_frame = if session.is_some() {
            engine.process_with_diagnostics(&event)
        } else {
            engine.process(&event)
        };
        let Some(frame) = engine_frame else {
            continue;
        };
        let added = frame
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::TangibleAdded(_)));
        for engine_event in &frame.events {
            match engine_event {
                EngineEvent::TangibleAdded(_) => println!(
                    "[Frame {frame_no}] figure detected ({} active)",
                    engine.tracked().len()
                ),
                EngineEvent::TangibleRemoved(_) if engine.tracked().is_empty() => {
                    println!("[Frame {frame_no}] figure lost");
                }
                EngineEvent::FieldPlaced { set, field } => {
                    println!("[Frame {frame_no}] field occupied: {set}.{field}")
                }
                EngineEvent::FieldLifted { set, field } => {
                    println!("[Frame {frame_no}] field free: {set}.{field}")
                }
                EngineEvent::FieldCountChanged { set, field, count } => {
                    println!("[Frame {frame_no}] field {set}.{field}: {count} contact(s)")
                }
                _ => {}
            }
        }

        let active = engine.tracked();
        if !active.is_empty() {
            detected_frames += 1;
            if detected_frames.is_multiple_of(50) || added {
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
            }
        }
        if let Some(session) = &mut session {
            session.record_frame(
                FrameNumber::new(frame_no),
                Milliseconds::from(Duration::from_secs_f64(offset)),
                &frame,
                engine.tracked(),
            );
        }
        frame_no += 1;
    }

    println!("\n--- summary ---");
    println!("frames total: {frame_no}");
    println!("frames with detection: {detected_frames}");
    if let Some(session) = session {
        let recorded = session.recorded_frames();
        let dropped_frames = session.dropped_frames();
        let dropped_events = session.dropped_events();
        session.finish().into_diagnostic()?;
        println!(
            "recording: {recorded} frames (dropped: {dropped_frames} frames, {dropped_events} events)"
        );
    }
    Ok(())
}
