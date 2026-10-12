use serde::Deserialize;
use serde::Serialize;
use tangible_objects_runtime::ProfileInfo;
use zvariant::Type;

/// A loaded figure profile, D-Bus-marshalled as `(usu)`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
