use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use tangible_objects_model::PatternMatch;
use tangible_objects_model::RotationRad;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TouchSet;

use super::pin_map::PinMap;

/// Persistent slot→pin bindings of all tracked tangibles
/// (ROTATION.md): each entry is solved once at spawn, reused for
/// rotation on every incumbent frame instead of the argmax
/// `rotation_of` estimate.
///
/// The map's lifecycle is the tracker lifecycle - spawn inserts,
/// refresh re-binds only when the genuine slot set changed, expiry
/// removes.
#[derive(Default)]
pub(crate) struct PinMaps(pub HashMap<TangibleId, PinMap>);

impl PinMaps {
    /// Correspondence-fitted rotation of the tangible's bound slots -
    /// `None` when no map exists or fewer than two bound slots are
    /// live (the caller keeps the match's own angle then).
    pub fn rotation_of(&self, id: TangibleId, touches: TouchSet<'_>) -> Option<RotationRad> {
        self.0.get(&id).and_then(|map| map.rotation(touches))
    }

    /// Syncs the tangible's binding to the frame's match: inserts a
    /// fresh binding when none exists, re-binds the genuine slots
    /// when the genuine slot set changed (mass slot churn) - claims
    /// always follow their current pairs inside `refresh`.
    pub fn sync(&mut self, id: TangibleId, pattern_match: &PatternMatch, touches: TouchSet<'_>) {
        let map = self
            .0
            .entry(id)
            .or_insert_with(|| PinMap::from_match(pattern_match, touches));
        if map.genuine_slots() != pattern_match.matched_slots(touches)
            && !pattern_match.correspondence.is_empty()
        {
            map.refresh_genuine(pattern_match, touches);
        }
        map.update_claimed(pattern_match, touches);
    }
}

impl Deref for PinMaps {
    type Target = HashMap<TangibleId, PinMap>;

    /// Maps deref to the underlying `HashMap` - `remove` on expiry.
    fn deref(&self) -> &HashMap<TangibleId, PinMap> {
        &self.0
    }
}

impl DerefMut for PinMaps {
    /// Mutable access to the underlying `HashMap`.
    fn deref_mut(&mut self) -> &mut HashMap<TangibleId, PinMap> {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PinMaps;
    use tangible_objects_model::ClaimedIndices;
    use tangible_objects_model::CombinationBuf;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternMatch;
    use tangible_objects_model::RefIndices;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn square() -> Vec<Vec2<Millimeters>> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 40.0),
        ]
    }

    fn touches(angle: f64, slots: &[u32]) -> Vec<TouchPoint> {
        let (sin, cos) = angle.sin_cos();
        square()
            .iter()
            .zip(slots)
            .map(|(p, &slot)| TouchPoint {
                slot: TouchPointSlot::new(slot),
                tracking_id: TrackingId::new(slot as i32),
                pos: Vec2::new(100.0 + p.x * cos - p.y * sin, 100.0 + p.x * sin + p.y * cos),
            })
            .collect()
    }

    fn solved_match(refs: &[Vec2<Millimeters>], touch_indices: Vec<usize>) -> PatternMatch {
        PatternMatch {
            pattern_id: PatternId::new(1),
            pattern_name: "quad".into(),
            center: Vec2::new(0.0, 0.0),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: CombinationBuf::from(touch_indices),
            correspondence: RefIndices::all(refs.len().into()),
            ref_points: Some(refs.to_vec().into()),
            partial: false,
            claimed_indices: ClaimedIndices::default(),
            spawn_stability: FrameCount::new(15),
        }
    }

    /// First `sync` inserts a fresh binding under the tangible's id.
    #[test]
    fn sync_inserts_on_first_match() {
        let mut maps = PinMaps::default();
        let id = TangibleId::new(1);
        let touch_set = touches(0.0, &[0, 1, 2, 3]);
        let pattern_match = solved_match(&square(), vec![0, 1, 2, 3]);
        maps.sync(id, &pattern_match, (&touch_set).into());
        assert_eq!(maps.get(&id).map(|m| m.genuine_slots().len()), Some(4));
    }

    /// `rotation_of` yields `None` for an unregistered tangible.
    #[test]
    fn rotation_of_none_for_unknown_tangible() {
        let maps = PinMaps::default();
        let touch_set = touches(0.0, &[0, 1, 2, 3]);
        assert!(
            maps.rotation_of(TangibleId::new(9), (&touch_set).into())
                .is_none()
        );
    }

    /// After `sync`, `rotation_of` fits the bound slots' planted
    /// rotation back out of the persisted map.
    #[test]
    fn rotation_of_reads_planted_angle() {
        let mut maps = PinMaps::default();
        let id = TangibleId::new(1);
        let spawn = touches(0.0, &[0, 1, 2, 3]);
        let pattern_match = solved_match(&square(), vec![0, 1, 2, 3]);
        maps.sync(id, &pattern_match, (&spawn).into());
        let moved = touches(0.3, &[0, 1, 2, 3]);
        let rotation = maps.rotation_of(id, (&moved).into());
        assert!(
            rotation.is_some_and(|r| (r.as_f64() - 0.3).abs() < 0.05),
            "synced map must recover the planted rotation"
        );
    }

    /// A match whose genuine slots changed re-binds the map -
    /// `sync` runs `refresh_genuine`, not just `update_claimed`.
    #[test]
    fn sync_rebinds_when_genuine_slots_change() {
        let mut maps = PinMaps::default();
        let id = TangibleId::new(1);
        let spawn = touches(0.0, &[0, 1, 2, 3]);
        let first = solved_match(&square(), vec![0, 1, 2, 3]);
        maps.sync(id, &first, (&spawn).into());
        // Same pattern re-bound on four different slots - the touch
        // indices stay positional; the slots behind them changed.
        let rebound = touches(0.0, &[4, 5, 6, 7]);
        let second = solved_match(&square(), vec![0, 1, 2, 3]);
        maps.sync(id, &second, (&rebound).into());
        let bound = maps.get(&id).map(|m| m.genuine_slots()).unwrap_or_default();
        assert!(bound.contains(&TouchPointSlot::new(4)));
        assert!(!bound.contains(&TouchPointSlot::new(0)));
    }
}
