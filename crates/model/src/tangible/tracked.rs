use super::TangibleId;
use crate::Millimeters;
use crate::PatternId;
use crate::RotationRad;
use crate::Vec2;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

/// A tangible figure tracked across frames with a smoothed pose.
///
/// Produced by the engine's `Tracker`; position and rotation are
/// exponentially smoothed over consecutive frames. Serializes to the
/// interface-facing flat shape `{id, pattern_id, pattern_name, center_x,
/// center_y, rotation_deg}` - millimeters and degrees, without the internal
/// `frames_since_match` hysteresis counter.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(complex))]
pub struct TrackedTangible {
    /// Stable ID of this tangible, assigned by the `Tracker` on creation.
    /// Distinguishes tangibles matching the same pattern.
    pub id: TangibleId,
    /// ID of the matched pattern.
    pub pattern_id: PatternId,
    /// Name of the matched figure profile.
    pub pattern_name: crate::PatternName,
    /// Smoothed centroid position.
    pub center: Vec2<Millimeters>,
    /// Smoothed orientation in radians.
    pub rotation_rad: RotationRad,
    /// Number of consecutive frames without a match for this tangible.
    /// Used for expiry hysteresis; internal bookkeeping, not exposed on
    /// interfaces.
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    pub frames_since_match: u32,
}

impl Serialize for TrackedTangible {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TrackedTangible", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("pattern_id", &self.pattern_id)?;
        state.serialize_field("pattern_name", &self.pattern_name)?;
        state.serialize_field("center_x", &self.center.x)?;
        state.serialize_field("center_y", &self.center.y)?;
        state.serialize_field("rotation_deg", &self.rotation_rad.to_degrees())?;
        state.end()
    }
}

#[cfg(feature = "async-graphql")]
#[async_graphql::ComplexObject]
impl TrackedTangible {
    /// Smoothed orientation in degrees.
    async fn rotation_deg(&self) -> f64 {
        self.rotation_rad.to_degrees()
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for TrackedTangible {
    const SIGNATURE: &'static zvariant::Signature = &zvariant::signature!("(uusddd)");
}
