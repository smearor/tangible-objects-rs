//! Field-set methods - `register_field_set`, `unregister_field_set`.

use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Registers an occupancy field set from a TOML string (same shape as
/// `--fields` files).
pub(crate) async fn register_field_set(
    service: &EngineService,
    toml_str: &str,
    owner: Owner,
    persistent: bool,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let set = toml::from_str::<FieldSet>(toml_str)
        .map_err(|e| fdo::Error::InvalidArgs(format!("invalid field set TOML: {e}")))?;
    let (command, rx) = ControlCommand::register(
        Registration::FieldSet(set),
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
        Ok(result) => result.map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the registration reply".to_string(),
        )),
    }
}

/// Removes a field set by name.
pub(crate) async fn unregister_field_set(
    service: &EngineService,
    name: FieldSetName,
) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterFieldSet { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
