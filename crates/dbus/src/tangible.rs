use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::TrackedTangible;
use zvariant::Type;

/// A currently tracked tangible with its smoothed pose, D-Bus-marshalled
/// as `(ussdddd)`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Tangible {
    /// Stable tracking ID for the object's lifetime on the surface.
    pub id: u32,
    /// Pattern ID of the matched figure profile.
    pub pattern_id: u32,
    /// Name of the matched figure profile.
    pub pattern_name: String,
    /// Smoothed center X in millimeters.
    pub center_x: f64,
    /// Smoothed center Y in millimeters.
    pub center_y: f64,
    /// Smoothed rotation in degrees.
    pub rotation_deg: f64,
}

impl From<&TrackedTangible> for Tangible {
    fn from(tangible: &TrackedTangible) -> Self {
        Self {
            id: tangible.id.as_u32(),
            pattern_id: tangible.pattern_id.as_u32(),
            pattern_name: tangible.pattern_name.clone(),
            center_x: tangible.center.x,
            center_y: tangible.center.y,
            rotation_deg: tangible.rotation_rad.to_degrees(),
        }
    }
}
