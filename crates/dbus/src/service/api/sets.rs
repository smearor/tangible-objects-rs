//! Set query methods - `sets`.

use tangible_objects_model::TangibleSetState;

use crate::service::engine_service::EngineService;

/// Tangible Object Sets joined with their live activation state.
pub(crate) async fn sets(service: &EngineService) -> Vec<TangibleSetState> {
    let snapshot = service.handle.snapshot();
    service
        .handle
        .sets()
        .iter()
        .map(|set| TangibleSetState::from((set, snapshot.active_sets.contains(&set.id))))
        .collect()
}
