//! Profile query methods - `list_profiles`.

use tangible_objects_model::Owner;
use tangible_objects_runtime::ProfileInfo;

use crate::service::engine_service::EngineService;

/// Loaded figure profiles - `owner` filters to one owning application
/// (`""` for all), `static_only` keeps only owner-less static profiles
/// (same semantics as the GraphQL `profiles` query).
pub(crate) async fn list_profiles(
    service: &EngineService,
    owner: &Owner,
    static_only: bool,
) -> Vec<ProfileInfo> {
    service
        .handle
        .profiles()
        .into_iter()
        .filter(|info| owner.as_str().is_empty() || info.owner.as_ref() == Some(owner))
        .filter(|info| !(static_only && info.owner.is_some()))
        .collect()
}
