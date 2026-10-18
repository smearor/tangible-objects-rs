use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

use crate::FrameCount;
use crate::Millimeters;
use crate::PatternId;
use crate::PatternName;
use crate::RotationRad;
use crate::TangibleId;
use crate::TrackedTangible;
use crate::Vec2;

/// Recorded view of a [`TrackedTangible`] within a [`crate::FrameRecord`].
///
/// Unlike the interface-facing serialization of [`TrackedTangible`] this
/// record keeps the internal `frames_since_match` hysteresis counter - raw
/// matches and stably tracked objects must be distinguishable in the
/// flappy-contact analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedRecord {
    /// Stable tracker ID of the tangible.
    pub id: TangibleId,
    /// ID of the matched pattern.
    pub pattern_id: PatternId,
    /// Name of the matched figure profile.
    pub pattern_name: PatternName,
    /// Smoothed centroid position.
    pub center: Vec2<Millimeters>,
    /// Smoothed orientation in radians; serialized in degrees as
    /// `rotation_deg`.
    pub rotation_rad: RotationRad,
    /// Consecutive frames without a match for this tangible.
    pub frames_since_match: FrameCount,
    /// Whether this tangible is an unconfirmed partial spawn
    /// (SPAWN_AS_PARTIAL.md). `#[serde(default)]` keeps older recordings
    /// loadable - recorded before pending existed, they show `false`.
    pub pending: bool,
}

impl From<&TrackedTangible> for TrackedRecord {
    fn from(tracked: &TrackedTangible) -> Self {
        Self {
            id: tracked.id,
            pattern_id: tracked.pattern_id,
            pattern_name: tracked.pattern_name.clone(),
            center: tracked.center,
            rotation_rad: tracked.rotation_rad,
            frames_since_match: tracked.frames_since_match,
            pending: tracked.pending,
        }
    }
}

impl Serialize for TrackedRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TrackedRecord", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("pattern_id", &self.pattern_id)?;
        state.serialize_field("pattern_name", &self.pattern_name)?;
        state.serialize_field("center", &self.center)?;
        state.serialize_field("rotation_deg", &self.rotation_rad.to_degrees())?;
        state.serialize_field("frames_since_match", &self.frames_since_match)?;
        state.serialize_field("pending", &self.pending)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TrackedRecord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            id: TangibleId,
            pattern_id: PatternId,
            pattern_name: String,
            center: Vec2<Millimeters>,
            rotation_deg: f64,
            frames_since_match: FrameCount,
            #[serde(default)]
            pending: bool,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            id: helper.id,
            pattern_id: helper.pattern_id,
            pattern_name: helper.pattern_name.into(),
            center: helper.center,
            rotation_rad: RotationRad::new(helper.rotation_deg.to_radians()),
            frames_since_match: helper.frames_since_match,
            pending: helper.pending,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TrackedRecord;
    use crate::DEFAULT_SPAWN_STABILITY;
    use crate::FrameCount;
    use crate::RotationRad;
    use crate::TangibleId;
    use crate::TrackedTangible;
    use crate::Vec2;

    fn tracked_tangible() -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(10),
            pattern_id: 5.into(),
            pattern_name: "TestFigur-5P".into(),
            center: Vec2::new(171.94, 80.84),
            rotation_rad: RotationRad::new(std::f64::consts::PI),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            frames_since_match: FrameCount::new(3),
        }
    }

    #[test]
    fn serializes_degrees_and_hysteresis() {
        let record = TrackedRecord::from(&tracked_tangible());
        let json = serde_json::to_value(&record).expect("serialize");

        assert_eq!(json["id"], 10);
        assert_eq!(json["pattern_id"], 5);
        assert_eq!(json["frames_since_match"], 3);
        assert!((json["rotation_deg"].as_f64().expect("deg") - 180.0).abs() < 1e-9);
        assert!(json["center"]["x"].is_number());
    }

    #[test]
    fn from_tracked_tangible_copies_all_fields() {
        let tracked = tracked_tangible();
        let record = TrackedRecord::from(&tracked);

        assert_eq!(record.id, tracked.id);
        assert_eq!(record.pattern_id, tracked.pattern_id);
        assert_eq!(record.pattern_name, tracked.pattern_name);
        assert_eq!(record.center, tracked.center);
        assert_eq!(record.rotation_rad, tracked.rotation_rad);
        assert_eq!(record.frames_since_match, tracked.frames_since_match);
    }
}
