use async_graphql::SimpleObject;
use tangible_objects_runtime::ProfileInfo;

/// A loaded figure profile.
#[derive(Debug, Clone, SimpleObject)]
pub struct Profile {
    /// Runtime pattern ID of the profile.
    pub pattern_id: u32,
    /// Human-readable figure name.
    pub name: String,
    /// Number of contact pins the figure must have.
    pub num_points: u32,
}

impl From<&ProfileInfo> for Profile {
    fn from(profile: &ProfileInfo) -> Self {
        Self {
            pattern_id: profile.pattern_id.as_u32(),
            name: profile.name.clone(),
            num_points: profile.num_points,
        }
    }
}
