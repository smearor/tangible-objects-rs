use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::TangiblePattern;

/// Static info about a loaded figure profile for query-style responses.
///
/// Carried by [`crate::EngineHandle`]; profiles are static for a session and
/// therefore live on the handle rather than in the per-frame snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct ProfileInfo {
    /// Runtime pattern ID assigned when the profile was loaded.
    pub pattern_id: PatternId,
    /// Human-readable figure name from the profile file.
    pub name: String,
    /// Number of contact pins the figure must have.
    pub num_points: u32,
    /// Owning application for runtime registrations; `None` for
    /// statically loaded profiles.
    pub owner: Option<Owner>,
}

#[cfg(feature = "zbus")]
impl zvariant::Type for ProfileInfo {
    const SIGNATURE: &'static zvariant::Signature = &zvariant::signature!("(usus)");
}

impl From<&TangiblePattern> for ProfileInfo {
    fn from(pattern: &TangiblePattern) -> Self {
        Self {
            pattern_id: pattern.id,
            name: pattern.name.clone(),
            num_points: pattern.num_points as u32,
            owner: None,
        }
    }
}
