//! Dead-zone methods - `register_dead_zone`, `unregister_dead_zone`.

use tangible_objects_model::DEFAULT_DEAD_ZONE_RADIUS;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Owner;
use tangible_objects_model::Registration;
use tangible_objects_model::Vec2;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Blacklists a circular spot on the surface (`x`/`y` in millimeters):
/// contacts inside `radius` millimeters are dropped at ingest - ghost
/// contacts never reach tracking, matching or fields. `radius <= 0`
/// picks the 10 mm default; `persistent` writes the zone to the
/// runtime store so it reloads on daemon start.
pub(crate) async fn register_dead_zone(
    service: &EngineService,
    name: DeadZoneName,
    x: f64,
    y: f64,
    radius: f64,
    owner: Owner,
    persistent: bool,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let zone = DeadZone::new(
        name,
        Vec2::<Millimeters>::new(x, y),
        if radius > 0.0 {
            Millimeters::new(radius)
        } else {
            DEFAULT_DEAD_ZONE_RADIUS
        },
    );
    let (command, rx) = ControlCommand::register(
        Registration::DeadZone(zone),
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

/// Removes a dead zone by name.
pub(crate) async fn unregister_dead_zone(
    service: &EngineService,
    name: DeadZoneName,
) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterDeadZone { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
