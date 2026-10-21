//! Tangible query methods - `list_tangibles`.

use tangible_objects_model::TrackedTangible;

use crate::service::engine_service::EngineService;

/// All currently tracked tangibles.
pub(crate) async fn list_tangibles(service: &EngineService) -> Vec<TrackedTangible> {
    service.handle.snapshot().tracked
}
