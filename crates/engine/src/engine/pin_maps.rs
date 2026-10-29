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
