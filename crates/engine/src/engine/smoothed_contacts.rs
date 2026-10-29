use std::collections::HashMap;

use tangible_objects_model::SmoothingAlpha;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TouchPointSlot;

use super::smoothed_contact::SmoothedContact;

/// Per-slot EMA smoothing state for the whole frame
/// (TRACKING_STABILIZATION.md): last smoothed position per slot,
/// keyed with the tracking ID so a new contact on the same slot
/// restarts at its raw position and stale state of a lifted contact
/// is dropped before blending.
#[derive(Default)]
pub(crate) struct SmoothedContacts(pub HashMap<TouchPointSlot, SmoothedContact>);

impl SmoothedContacts {
    /// Advances the smoothing state one frame.
    ///
    /// Entries of slots whose owner changed (stale state of a lifted
    /// contact) are dropped; every live touch gets its smoothed
    /// position assigned in place. `alpha <= 0` or `>= 1` leaves the
    /// frame untouched.
    pub fn step(&mut self, alpha: SmoothingAlpha, touches: &mut [TouchPoint]) {
        if !alpha.is_effective() {
            return;
        }
        self.0.retain(|slot, state| {
            touches
                .iter()
                .any(|t| t.slot == *slot && t.tracking_id == state.tracking_id)
        });
        for touch in touches.iter_mut() {
            let smoothed = self
                .0
                .get(&touch.slot)
                .map(|state| state.blend(alpha, touch.tracking_id, touch.pos))
                .unwrap_or(touch.pos);
            self.0.insert(
                touch.slot,
                SmoothedContact::new(touch.tracking_id, smoothed),
            );
            touch.pos = smoothed;
        }
    }
}
