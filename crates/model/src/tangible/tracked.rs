use super::TangibleId;
use crate::FrameCount;
use crate::Millimeters;
use crate::PatternId;
use crate::PatternName;
use crate::RotationRad;
use crate::Vec2;
use serde::Deserialize;
use serde::Deserializer;
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
    pub pattern_name: PatternName,
    /// Smoothed centroid position.
    pub center: Vec2<Millimeters>,
    /// Smoothed orientation in radians.
    pub rotation_rad: RotationRad,
    /// Whether this tangible was spawned by a partial match and has not
    /// yet confirmed (SPAWN_AS_PARTIAL.md). Pending tangibles reserve
    /// their slots but emit no events until the spawn-stability window
    /// or an exact match confirms them.
    pub pending: bool,
    /// Consecutive match frames the pending spawn has held so far;
    /// internal bookkeeping, not exposed on interfaces.
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    pub pending_frames: FrameCount,
    /// Confirmation threshold carried from the pattern; internal
    /// bookkeeping, not exposed on interfaces.
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    pub spawn_stability: FrameCount,
    /// Number of consecutive frames without a match for this tangible.
    /// Used for expiry hysteresis; internal bookkeeping, not exposed on
    /// interfaces.
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    pub frames_since_match: FrameCount,
}

impl Serialize for TrackedTangible {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TrackedTangible", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("pattern_id", &self.pattern_id)?;
        state.serialize_field("pattern_name", &self.pattern_name)?;
        state.serialize_field("center_x", &self.center.x)?;
        state.serialize_field("center_y", &self.center.y)?;
        state.serialize_field("rotation_deg", &self.rotation_rad.to_degrees())?;
        state.serialize_field("pending", &self.pending)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TrackedTangible {
    /// Accepts both wire shapes: the GraphQL `SimpleObject` form
    /// (`center {x,y}` + `rotationRad`, camelCase keys) and the legacy
    /// flat serde form (`center_x`/`center_y`/`rotation_deg`).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Helper {
            id: TangibleId,
            pattern_id: PatternId,
            pattern_name: PatternName,
            center: Option<Vec2<Millimeters>>,
            #[serde(default, alias = "center_x")]
            center_x: Option<f64>,
            #[serde(default, alias = "center_y")]
            center_y: Option<f64>,
            rotation_rad: Option<RotationRad>,
            #[serde(default)]
            rotation_deg: Option<f64>,
            #[serde(default)]
            pending: bool,
        }
        let h = Helper::deserialize(deserializer)?;
        let center = h.center.unwrap_or_else(|| {
            Vec2::new(
                h.center_x.unwrap_or_default(),
                h.center_y.unwrap_or_default(),
            )
        });
        let rotation_rad = h
            .rotation_rad
            .or_else(|| h.rotation_deg.map(|deg| RotationRad(deg.to_radians())))
            .unwrap_or(RotationRad(0.0));
        Ok(Self {
            id: h.id,
            pattern_id: h.pattern_id,
            pattern_name: h.pattern_name,
            center,
            rotation_rad,
            pending: h.pending,
            pending_frames: FrameCount::default(),
            spawn_stability: FrameCount::default(),
            frames_since_match: FrameCount::default(),
        })
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
