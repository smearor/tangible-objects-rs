use crate::FrameCount;
use crate::Owner;
use crate::PatternId;
use crate::PatternName;
use crate::PinCount;
use crate::StageName;
use crate::TangiblePattern;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// Static info about a loaded figure profile for query-style responses.
///
/// Carried by [`crate::EngineHandle`]; profiles are static for a session and
/// therefore live on the handle rather than in the per-frame snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "ProfileInfo",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct ProfileInfo {
    /// Runtime pattern ID assigned when the profile was loaded.
    #[serde(alias = "pattern_id")]
    pub pattern_id: PatternId,
    /// Human-readable figure name from the profile file.
    pub name: PatternName,
    /// Number of contact pins the figure must have.
    #[serde(alias = "num_points")]
    pub num_points: PinCount,
    /// Owning application for runtime registrations; `None` for
    /// statically loaded profiles.
    pub owner: Option<Owner>,
    /// Whether a partial match may spawn this figure
    /// (SPAWN_AS_PARTIAL.md).
    #[serde(alias = "spawn_on_partial")]
    pub spawn_on_partial: bool,
    /// Frames a partial spawn must hold before confirmation.
    #[serde(alias = "spawn_stability")]
    pub spawn_stability: FrameCount,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md); `None` = the general stage.
    pub stage: Option<StageName>,
}

#[cfg(feature = "zbus")]
impl zvariant::Type for ProfileInfo {
    const SIGNATURE: &'static zvariant::Signature = &zvariant::signature!("(ustsbt)");
}

impl From<&TangiblePattern> for ProfileInfo {
    fn from(pattern: &TangiblePattern) -> Self {
        Self {
            pattern_id: pattern.id,
            name: pattern.name.clone(),
            num_points: pattern.num_points,
            owner: None,
            spawn_on_partial: pattern.spawn_on_partial,
            spawn_stability: pattern.spawn_stability,
            stage: pattern.stage.clone(),
        }
    }
}
