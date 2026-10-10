use super::TangibleId;
use crate::Millimeters;
use crate::PatternId;
use crate::RotationRad;
use crate::Vec2;

/// A tangible figure tracked across frames with a smoothed pose.
///
/// Produced by the engine's `Tracker`; position and rotation are
/// exponentially smoothed over consecutive frames.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedTangible {
    /// Stable ID of this tangible, assigned by the `Tracker` on creation.
    /// Distinguishes tangibles matching the same pattern.
    pub id: TangibleId,
    /// ID of the matched pattern.
    pub pattern_id: PatternId,
    /// Name of the matched figure profile.
    pub pattern_name: String,
    /// Smoothed centroid position.
    pub center: Vec2<Millimeters>,
    /// Smoothed orientation in radians.
    pub rotation_rad: RotationRad,
    /// Number of consecutive frames without a match for this tangible.
    /// Used for expiry hysteresis.
    pub frames_since_match: u32,
}
