//! Training methods - `train_profile`, `train_spell`.

use tangible_objects_model::PatternName;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Starts a training session capturing a new profile with `name`.
pub(crate) async fn train_profile(service: &EngineService, name: PatternName) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::TrainProfile(name))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Starts a spell training session (SPELLS.md's learn-by-repetition):
/// the next completed gesture episodes become the trained spell's
/// template strokes and corridor; the finished spell registers
/// persistently. `arming_toml` is the `[arming]` table's TOML (e.g.
/// `type = "five-finger-tap"`); an empty string means `free`.
pub(crate) async fn train_spell(
    service: &EngineService,
    name: SpellName,
    arming_toml: &str,
) -> fdo::Result<()> {
    let arming = if arming_toml.is_empty() {
        SpellArming::Free
    } else {
        toml::from_str::<SpellArming>(arming_toml)
            .map_err(|e| fdo::Error::InvalidArgs(format!("invalid arming TOML: {e}")))?
    };
    service
        .handle
        .send_command(ControlCommand::TrainSpell { name, arming })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
