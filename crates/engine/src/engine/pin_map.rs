//! `PinMap` - a tracked tangible's persistent slot→reference-pin
//! binding (ROTATION.md). Solved once at spawn, re-solved only when
//! the genuine slot set changes; claimed pins follow their claim
//! frame by frame.

use std::collections::HashMap;

use tangible_objects_model::CombinationBuf;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::RefIndex;
use tangible_objects_model::RefPoints;
use tangible_objects_model::RotationRad;
use tangible_objects_model::SlotSet;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_model::TouchSet;

/// Slot→pin identity of one tracked tangible, plus the pattern's
/// reference geometry (stored at spawn - no pattern lookup needed on
/// the incumbent path).
pub(crate) struct PinMap {
    /// The pattern's reference pin positions.
    refs: RefPoints,
    /// Genuinely matched slots → reference pin index. Rewritten only
    /// on spawn and when the genuine slot set changed (map-refresh
    /// rule, ROTATION.md).
    genuine: HashMap<TouchPointSlot, RefIndex>,
    /// Completion-claimed slots → the pin they currently fill.
    /// Rebuilt from `claimed_indices` every frame - a wandering claim
    /// follows its pair without re-solving.
    claimed: HashMap<TouchPointSlot, RefIndex>,
}

impl PinMap {
    /// Builds the map from a solved match: `correspondence` binds the
    /// genuine slots, `claimed_indices` the claimed ones. Empty maps
    /// when the match carries no correspondence/ref_points.
    pub(crate) fn from_match(pattern_match: &PatternMatch, touches: TouchSet<'_>) -> Self {
        let mut map = Self {
            refs: pattern_match.ref_points.clone().unwrap_or_default(),
            genuine: pattern_match.genuine_map(touches),
            claimed: HashMap::new(),
        };
        map.update_claimed(pattern_match, touches);
        map
    }

    /// Rebuilds the claimed half from the match's `Claim`s
    /// (claim churn follows the current frame's bindings).
    pub(crate) fn update_claimed(&mut self, pattern_match: &PatternMatch, touches: TouchSet<'_>) {
        self.claimed = pattern_match
            .claimed_indices
            .as_slice()
            .iter()
            .map(|claim| (touches[claim.touch_index].slot, claim.ref_index))
            .collect();
    }

    /// Rewrites the genuine half from a fresh solved match
    /// (map-refresh rule: genuine slots changed → re-bind).
    pub(crate) fn refresh_genuine(&mut self, pattern_match: &PatternMatch, touches: TouchSet<'_>) {
        self.refs = pattern_match.ref_points.clone().unwrap_or_default();
        self.genuine = pattern_match.genuine_map(touches);
    }

    /// The slots the genuine half binds - compared to
    /// `matched_slots` for the refresh rule.
    pub(crate) fn genuine_slots(&self) -> SlotSet {
        self.genuine.keys().copied().collect()
    }

    /// Rotation of the mapped slots' current positions against the
    /// stored reference frame - `similarity_transform` over all
    /// bound slots present in `touches`. `None` when fewer than two
    /// bound slots are live (keep the last good angle then).
    pub(crate) fn rotation(&self, touches: TouchSet<'_>) -> Option<RotationRad> {
        let (measured, mapping): (Vec<_>, CombinationBuf) = self
            .genuine
            .iter()
            .chain(&self.claimed)
            .filter_map(|(slot, ref_index)| {
                touches
                    .iter()
                    .find(|t| t.slot == *slot)
                    .map(|touch| (touch.pos, ref_index.as_usize()))
            })
            .unzip();
        self.refs
            .similarity_transform((&measured).into(), mapping.as_combination())
            .map(|transform| transform.angle)
    }
}

#[cfg(test)]
mod tests {
    use super::PinMap;
    use tangible_objects_model::ClaimedIndices;
    use tangible_objects_model::CombinationBuf;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternMatch;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    /// Touch points `slot`, `(x, y)` - a fixed square under `angle`.
    fn touches(angle: f64) -> Vec<TouchPoint> {
        let (sin, cos) = angle.sin_cos();
        [(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)]
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| TouchPoint {
                slot: TouchPointSlot::new(i as u32),
                tracking_id: TrackingId::new(i as i32),
                pos: Vec2::new(100.0 + x * cos - y * sin, 100.0 + x * sin + y * cos),
            })
            .collect()
    }

    fn solved_match(refs: &[Vec2<Millimeters>]) -> PatternMatch {
        PatternMatch {
            pattern_id: PatternId::new(1),
            pattern_name: "quad".into(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: CombinationBuf::from(vec![0, 1, 2, 3]),
            correspondence: vec![0.into(), 1.into(), 2.into(), 3.into()].into(),
            ref_points: Some(refs.to_vec().into()),
            partial: false,
            claimed_indices: ClaimedIndices::default(),
            spawn_stability: FrameCount::new(15),
        }
    }

    /// The persisted map reproduces the planted rotation of the
    /// bound slots (ROTATION.md testing section).
    #[test]
    fn rotation_reads_planted_angle() {
        let refs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 40.0),
        ];
        let angle = 0.7;
        let touch_points = touches(angle);
        let map = PinMap::from_match(&solved_match(&refs), (&touch_points).into());
        let rotation = map
            .rotation((&touch_points).into())
            .expect("identity-mapped square solves");
        assert!(
            (rotation.as_f64() - angle).abs() < 0.05,
            "map rotation {}, planted {angle}",
            rotation.as_f64()
        );
    }

    /// Fewer than two bound slots gives no angle - the caller keeps
    /// the last good value then.
    #[test]
    fn rotation_none_under_two_slots() {
        let refs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 40.0),
        ];
        let mut match_ = solved_match(&refs);
        match_.touch_indices = CombinationBuf::from(vec![0]);
        match_.correspondence = vec![0.into()].into();
        let touch_points = touches(0.0);
        let map = PinMap::from_match(&match_, (&touch_points).into());
        assert_eq!(map.genuine_slots().len(), 1);
        assert!(map.rotation((&touch_points).into()).is_none());
    }
}
