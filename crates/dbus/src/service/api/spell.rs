//! Spell methods - `register_spell`, `unregister_spell`.

use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::Registration;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellName;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Registers a spell gesture from a TOML string (same shape as
/// `spells.d` files - `name`, `fingers`, `corridor`, `[[strokes]]`
/// plus the `[arming]` section, SPELLS.md).
pub(crate) async fn register_spell(
    service: &EngineService,
    toml_str: &str,
    owner: Owner,
    persistent: bool,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let spell = toml::from_str::<Spell>(toml_str)
        .map_err(|e| fdo::Error::InvalidArgs(format!("invalid spell TOML: {e}")))?;
    let (command, rx) = ControlCommand::register(
        Registration::Spell(spell),
        Some(service.session_owner(owner, header)),
        if persistent {
            Lifetime::Persistent
        } else {
            Lifetime::Session
        },
    );
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|_| ())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the registration reply".to_string(),
        )),
    }
}

/// Removes a registered spell by name.
pub(crate) async fn unregister_spell(service: &EngineService, name: SpellName) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterSpell { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
