//! Completion methods - `completion`, `set_completion`.

use tangible_objects_model::CompletionConfig;
use tangible_objects_model::CompletionMode;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Active missing-pin completion configuration as a flat tuple
/// (`radius_factor`, `max_missing`, `persist`, `mode`,
/// `affinity_factor`, `smoothing_alpha`) - empty strings are
/// impossible here; when nothing was published yet all values are
/// defaults.
pub(crate) async fn completion(service: &EngineService) -> (f64, u32, bool, String, f64, f64) {
    let config = service.handle.snapshot().completion.unwrap_or_default();
    (
        config.radius_factor,
        config.max_missing,
        config.persist,
        config.mode.to_string(),
        config.affinity_factor.as_f64(),
        config.smoothing_alpha.as_f64(),
    )
}

/// Replaces the missing-pin completion configuration (`mode` is
/// `match` or `exact`; `max_missing` must be 1 or 2; `affinity`
/// widens incumbent tolerances, `smoothing` is the position EMA
/// alpha).
pub(crate) async fn set_completion(
    service: &EngineService,
    radius_factor: f64,
    max_missing: u32,
    persist: bool,
    mode: &str,
    affinity: f64,
    smoothing: f64,
) -> fdo::Result<()> {
    let mode = mode
        .parse::<CompletionMode>()
        .map_err(|_| fdo::Error::InvalidArgs("mode must be one of: match, exact".to_string()))?;
    if !(1..=2).contains(&max_missing) {
        return Err(fdo::Error::InvalidArgs(
            "max_missing must be 1 or 2".to_string(),
        ));
    }
    let config = CompletionConfig {
        radius_factor,
        max_missing,
        persist,
        mode,
        affinity_factor: affinity.into(),
        smoothing_alpha: smoothing.into(),
    };
    service
        .handle
        .send_command(ControlCommand::SetCompletion(config))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
