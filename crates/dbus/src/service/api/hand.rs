//! Hand methods - `register_hand_profile`,
//! `unregister_hand_profile`.

use tangible_objects_model::HandName;
use tangible_objects_model::HandProfile;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Registers a handprint from a TOML string (same shape as
/// `hands.d` files - `name`, `tolerance`, `points`,
/// SPELLS.md's per-user spells).
pub(crate) async fn register_hand_profile(
    service: &EngineService,
    toml_str: &str,
    owner: Owner,
    persistent: bool,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let hand = toml::from_str::<HandProfile>(toml_str)
        .map_err(|e| fdo::Error::InvalidArgs(format!("invalid hand profile TOML: {e}")))?;
    let (command, rx) = ControlCommand::register(
        Registration::HandProfile(hand),
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

/// Removes a registered handprint by name.
pub(crate) async fn unregister_hand_profile(
    service: &EngineService,
    name: HandName,
) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterHandProfile { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
