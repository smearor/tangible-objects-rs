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

#[cfg(test)]
mod tests {
    use super::SmoothedContacts;
    use tangible_objects_model::SmoothingAlpha;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn touch(slot: u32, tracking: i32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(tracking),
            pos: Vec2::new(x, y),
        }
    }

    /// The second frame of a stable contact EMA-blends its position.
    #[test]
    fn step_blends_continuing_contact() {
        let mut contacts = SmoothedContacts::default();
        let alpha = SmoothingAlpha::new(0.5);
        let mut touches = vec![touch(0, 1, 0.0, 0.0)];
        contacts.step(alpha, &mut touches);
        assert_eq!(touches[0].pos, Vec2::new(0.0, 0.0), "first frame is raw");
        let mut touches = vec![touch(0, 1, 10.0, 20.0)];
        contacts.step(alpha, &mut touches);
        assert_eq!(touches[0].pos, Vec2::new(5.0, 10.0));
    }

    /// A slot reuse by a new contact restarts at the raw position -
    /// the previous owner's state must not leak.
    #[test]
    fn step_restarts_on_slot_reuse() {
        let mut contacts = SmoothedContacts::default();
        let alpha = SmoothingAlpha::new(0.5);
        let mut touches = vec![touch(0, 1, 0.0, 0.0)];
        contacts.step(alpha, &mut touches);
        let mut touches = vec![touch(0, 2, 10.0, 0.0)];
        contacts.step(alpha, &mut touches);
        assert_eq!(
            touches[0].pos,
            Vec2::new(10.0, 0.0),
            "a new tracking id must restart at the raw position"
        );
    }

    /// The stale entry of a lifted contact is dropped - the map keeps
    /// only slots still live under the same owner.
    #[test]
    fn step_drops_lifted_slots() {
        let mut contacts = SmoothedContacts::default();
        let alpha = SmoothingAlpha::new(0.5);
        let mut touches = vec![touch(0, 1, 0.0, 0.0)];
        contacts.step(alpha, &mut touches);
        let mut touches = vec![touch(1, 2, 50.0, 50.0)];
        contacts.step(alpha, &mut touches);
        assert_eq!(contacts.0.len(), 1);
        assert!(contacts.0.contains_key(&TouchPointSlot::new(1)));
    }

    /// `alpha >= 1` leaves both the frame and the map untouched.
    #[test]
    fn step_noop_on_ineffective_alpha() {
        let mut contacts = SmoothedContacts::default();
        let mut touches = vec![touch(0, 1, 10.0, 20.0)];
        contacts.step(SmoothingAlpha::new(1.0), &mut touches);
        assert_eq!(touches[0].pos, Vec2::new(10.0, 20.0));
        assert!(contacts.0.is_empty());
    }
}
