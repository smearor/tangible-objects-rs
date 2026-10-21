//! Recording methods - `recording_status`, `start_recording`,
//! `stop_recording`, `set_record_label`.

use std::path::PathBuf;

use tangible_objects_model::RecordLabel;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Live recording status as a flat key-value list; empty while no
/// session is or was active.
pub(crate) async fn recording_status(service: &EngineService) -> Vec<(String, String)> {
    let status = service.handle.snapshot().recording;
    let Some(status) = status else {
        return Vec::new();
    };
    vec![
        ("active".to_string(), status.active.to_string()),
        (
            "label".to_string(),
            status
                .label
                .map(|label| label.to_string())
                .unwrap_or_default(),
        ),
        (
            "recorded_frames".to_string(),
            status.recorded_frames.to_string(),
        ),
        (
            "dropped_frames".to_string(),
            status.dropped_frames.to_string(),
        ),
        (
            "dropped_events".to_string(),
            status.dropped_events.to_string(),
        ),
    ]
}

/// Starts a recording session writing `<base_path>.jsonl` +
/// `<base_path>.evtest`; an empty `label` records unlabeled.
pub(crate) async fn start_recording(
    service: &EngineService,
    base_path: &str,
    label: &str,
) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::StartRecording {
            base_path: PathBuf::from(base_path),
            label: (!label.is_empty()).then(|| RecordLabel::new(label)),
        })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Stops the active recording session.
pub(crate) async fn stop_recording(service: &EngineService) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::StopRecording)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Sets the ground-truth label for subsequent recorded frames; an empty
/// string clears it.
pub(crate) async fn set_record_label(service: &EngineService, label: &str) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetRecordLabel(
            (!label.is_empty()).then(|| RecordLabel::new(label)),
        ))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
