//! Filter methods - `set_filter_mode`.

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Switches the filter proxy's suppression policy
/// (`off`/`immediate`/`gated`/`guarded`); `stability > 0` overrides
/// the consecutive-matched-frames gate, `0` keeps it. `stage`
/// narrows the switch to that scope's tangible stage, `""` applies
/// to all.
pub(crate) async fn set_filter_mode(
    service: &EngineService,
    mode: &str,
    stability: FrameCount,
    stage: StageName,
) -> fdo::Result<()> {
    let mode = mode.parse::<FilterMode>().map_err(|_| {
        fdo::Error::InvalidArgs(
            "mode must be one of: off, immediate, gated, guarded, tangibles-only".to_string(),
        )
    })?;
    let stability = (stability.as_u64() > 0).then_some(stability);
    let stage = (!stage.as_str().is_empty()).then_some(stage);
    service
        .handle
        .send_command(ControlCommand::SetFilterMode {
            mode,
            stability,
            stage,
        })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
