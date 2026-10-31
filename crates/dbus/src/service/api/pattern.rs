//! Pattern methods - `register_pattern`, `update_profile`,
//! `unregister_pattern`.

use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Registration;
use tangible_objects_model::StageName;
use tangible_objects_model::Vec2;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Validates the flat pin list into `Vec2<Millimeters>` pairs and
/// fills the spawn/stage options - shared by `register_pattern` and
/// `update_profile`.
fn build_profile(
    name: &PatternName,
    points: &[f64],
    tolerance: PatternTolerance,
    spawn_on_partial: bool,
    spawn_stability: FrameCount,
    stage: &StageName,
) -> fdo::Result<ProfileFile> {
    if !points.len().is_multiple_of(2) || points.is_empty() {
        return Err(fdo::Error::InvalidArgs(
            "points must be a non-empty even-length flat list".to_string(),
        ));
    }
    let points: Vec<Vec2<Millimeters>> = points
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| Vec2::new(pair[0], pair[1]))
        .collect();
    let mut profile = ProfileFile::from_points(name.as_str(), &points, tolerance);
    profile.spawn_on_partial = spawn_on_partial;
    profile.stage = (!stage.as_str().is_empty()).then(|| stage.clone());
    if spawn_stability.as_u64() > 0 {
        profile.spawn_stability = spawn_stability;
    }
    Ok(profile)
}

/// Registers a figure profile at runtime: `points` is the flat pin
/// list `[x0, y0, x1, y1, ...]` in mm. `spawn_on_partial` allows
/// partial spawns (SPAWN_AS_PARTIAL.md); `spawn_stability = 0` keeps
/// the profile default. Queued to the device loop; fingerprint
/// collisions are rejected there - verify via `list_profiles`.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn register_pattern(
    service: &EngineService,
    name: PatternName,
    points: Vec<f64>,
    tolerance: PatternTolerance,
    owner: Owner,
    persistent: bool,
    spawn_on_partial: bool,
    spawn_stability: FrameCount,
    stage: StageName,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let profile = build_profile(
        &name,
        &points,
        tolerance,
        spawn_on_partial,
        spawn_stability,
        &stage,
    )?;
    let (command, rx) = ControlCommand::register(
        Registration::Pattern(profile),
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

/// Replaces a registered pattern's geometry while keeping its ID.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_profile(
    service: &EngineService,
    id: PatternId,
    name: PatternName,
    points: Vec<f64>,
    tolerance: PatternTolerance,
    spawn_on_partial: bool,
    spawn_stability: FrameCount,
    stage: StageName,
) -> fdo::Result<()> {
    let profile = build_profile(
        &name,
        &points,
        tolerance,
        spawn_on_partial,
        spawn_stability,
        &stage,
    )?;
    service
        .handle
        .send_command(ControlCommand::UpdateProfile {
            id,
            profile: Box::new(profile),
        })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Removes a dynamically registered pattern by ID.
pub(crate) async fn unregister_pattern(service: &EngineService, id: PatternId) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterPattern { id })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
