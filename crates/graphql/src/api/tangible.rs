use async_graphql::ID;
use async_graphql::SimpleObject;
use tangible_objects_model::TrackedTangible;

use super::Point;

/// A currently tracked tangible with its smoothed pose.
#[derive(Debug, Clone, SimpleObject)]
pub struct Tangible {
    /// Stable tracking ID for the object's lifetime on the surface.
    pub id: ID,
    /// Pattern ID of the matched figure profile.
    pub pattern_id: u32,
    /// Name of the matched figure profile.
    pub pattern_name: String,
    /// Smoothed center position in millimeters.
    pub center: Point,
    /// Smoothed rotation in radians.
    pub rotation_rad: f64,
    /// Smoothed rotation in degrees.
    pub rotation_deg: f64,
}

impl From<&TrackedTangible> for Tangible {
    fn from(tangible: &TrackedTangible) -> Self {
        Self {
            id: ID::from(tangible.id.to_string()),
            pattern_id: tangible.pattern_id.as_u32(),
            pattern_name: tangible.pattern_name.clone(),
            center: tangible.center.into(),
            rotation_rad: tangible.rotation_rad.as_f64(),
            rotation_deg: tangible.rotation_rad.to_degrees(),
        }
    }
}
