//! Metrics methods - `metrics`, `reset_metrics`.

use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Session metrics as JSON (uptime, re-ID churn, match quality,
/// per-slot contact stability).
pub(crate) async fn metrics(service: &EngineService) -> String {
    serde_json::to_string_pretty(&service.handle.snapshot().metrics)
        .unwrap_or_else(|_| "{}".to_string())
}

/// Resets the session metric counters.
pub(crate) async fn reset_metrics(service: &EngineService) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::ResetMetrics)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
