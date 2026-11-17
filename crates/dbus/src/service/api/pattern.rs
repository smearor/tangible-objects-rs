//! Pattern methods - `register_pattern`, `update_profile`,
//! `unregister_pattern`.

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Registration;
use tangible_objects_model::StageName;
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
        Ok(result) => result
            .map(|_| ())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the registration reply".to_string(),
        )),
    }
}

/// Compares a candidate profile against the registered patterns - the
/// `find_profile` check applications run before registering:
/// `(found, pattern_id, identical, owner)` tells the caller whether
/// the profile is already loaded and the same (`identical` flags exact
/// fingerprint equality), plus who owns it (`""` = owner-less,
/// claimable via `claim_profile`).
pub(crate) async fn find_profile(
    service: &EngineService,
    name: PatternName,
    points: Vec<f64>,
    tolerance: PatternTolerance,
) -> fdo::Result<(bool, u32, bool, String)> {
    let profile = build_profile(
        &name,
        &points,
        tolerance,
        false,
        FrameCount::new(0),
        &"".into(),
    )?;
    let (command, rx) = ControlCommand::lookup_profile(profile);
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(lookup) => Ok(match lookup.matching {
            Some(matching) => (
                true,
                matching.pattern_id.as_u32(),
                matching.identical,
                matching
                    .owner
                    .map(|owner| owner.as_str().to_string())
                    .unwrap_or_default(),
            ),
            None => (false, 0, false, String::new()),
        }),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the profile lookup reply".to_string(),
        )),
    }
}

/// Claims an owner-less profile for `owner` - an empty `owner`
/// defaults to the caller's unique bus name like `register_pattern`.
/// The profile is addressed by ID - `find_profile`'s result carries
/// it; profile names are not unique. Returns the claimed pattern ID;
/// a profile owned by a different application is rejected.
pub(crate) async fn claim_profile(
    service: &EngineService,
    id: PatternId,
    owner: Owner,
    header: &Header<'_>,
) -> fdo::Result<u32> {
    let (command, rx) = ControlCommand::claim_profile(id, service.session_owner(owner, header));
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|claimed| claimed.pattern_id.as_u32())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the claim reply".to_string(),
        )),
    }
}

/// Releases `owner`'s claim on a profile - an empty `owner` defaults
/// to the caller's unique bus name. Returns the released pattern ID;
/// the profile stays registered and keeps matching.
pub(crate) async fn release_profile(
    service: &EngineService,
    id: PatternId,
    owner: Owner,
    header: &Header<'_>,
) -> fdo::Result<u32> {
    let (command, rx) = ControlCommand::release_profile(id, service.session_owner(owner, header));
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|released| released.pattern_id.as_u32())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the release reply".to_string(),
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
