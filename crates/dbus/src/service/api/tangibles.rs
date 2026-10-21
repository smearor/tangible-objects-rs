//! Tangible query methods - `list_tangibles`, `touch_points`,
//! `get_pose`.

use tangible_objects_model::TangibleId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// All currently tracked tangibles.
pub(crate) async fn list_tangibles(service: &EngineService) -> Vec<TrackedTangible> {
    service.handle.snapshot().tracked
}

/// Active touch points of the last processed frame.
pub(crate) async fn touch_points(service: &EngineService) -> Vec<TouchPoint> {
    service.handle.snapshot().touches
}

/// Center `(x, y)` and rotation (radians) of one tangible by tracking ID.
pub(crate) async fn get_pose(
    service: &EngineService,
    id: TangibleId,
) -> fdo::Result<(f64, f64, f64)> {
    let snapshot = service.handle.snapshot();
    let Some(tangible) = snapshot.tracked.iter().find(|t| t.id == id) else {
        return Err(fdo::Error::Failed(format!(
            "no tangible with id {}",
            id.as_u32()
        )));
    };
    Ok((
        tangible.center.x,
        tangible.center.y,
        tangible.rotation_rad.as_f64(),
    ))
}
