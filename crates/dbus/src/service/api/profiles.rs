//! Profile query methods - `list_profiles`.

use tangible_objects_runtime::ProfileInfo;

use crate::service::engine_service::EngineService;

/// Loaded figure profiles.
pub(crate) async fn list_profiles(service: &EngineService) -> Vec<ProfileInfo> {
    service.handle.profiles()
}
