use std::collections::HashMap;
use std::collections::HashSet;

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::ABS_MT_POSITION_Y;
use tangible_objects_model::ABS_MT_SLOT;
use tangible_objects_model::ABS_MT_TRACKING_ID;
use tangible_objects_model::Calibration;
use tangible_objects_model::EventType;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_model::TrackingId;
use tangible_objects_model::Vec2;

/// Multitouch slot state machine for the Linux type B protocol.
///
/// Tracks every contact via `ABS_MT_SLOT` / `ABS_MT_TRACKING_ID` and updates
/// positions on `ABS_MT_POSITION_X/Y`. When a [`Calibration`] is set, raw
/// device units are converted to millimeters on the fly.
pub struct TouchState {
    /// Active contacts keyed by slot - inserted on `TRACKING_ID >= 0`,
    /// removed on lift-off (`-1`).
    slots: HashMap<TouchPointSlot, TouchPoint>,
    /// Slot addressed by subsequent `ABS_MT_*` events (the kernel
    /// delivers events per slot, addressed by `ABS_MT_SLOT`).
    current_slot: TouchPointSlot,
    /// Raw-to-millimeter conversion for position updates; `None` leaves
    /// positions in raw device units.
    calibration: Option<Calibration>,
    /// A `SYN_DROPPED` marker opened an authoritative resync frame:
    /// the kernel's full slot state follows, and every slot absent
    /// from it must be lifted at the next `SYN_REPORT`.
    resync_pending: bool,
    /// Slots addressed since the resync marker - survivors of the
    /// reconcile.
    updated: HashSet<TouchPointSlot>,
}

impl TouchState {
    /// Creates an empty touch state without calibration.
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
            current_slot: TouchPointSlot::new(0),
            calibration: None,
            resync_pending: false,
            updated: HashSet::new(),
        }
    }

    /// Sets the device calibration used for raw-to-mm conversion.
    pub fn set_calibration(&mut self, calibration: Calibration) {
        self.calibration = Some(calibration);
    }

    /// Feeds a single raw event into the state machine.
    pub fn process(&mut self, event: &RawEvent) {
        // SYN_DROPPED resync (kernel queue overflow): the frame after
        // the marker carries the full kernel slot state - any held
        // contact absent from it is dropped state and lifted at the
        // frame's SYN_REPORT.
        if event.is_syn_dropped() {
            self.resync_pending = true;
            self.updated.clear();
            return;
        }
        if event.is_syn_report() {
            if self.resync_pending {
                self.slots.retain(|slot, _| self.updated.contains(slot));
                self.resync_pending = false;
            }
            return;
        }
        if event.event_type == EventType::Abs {
            if self.resync_pending && event.code != ABS_MT_SLOT {
                // Slot-targeted data marks the already-selected slot -
                // ABS_MT_SLOT itself must not mark: current_slot still
                // holds the previous slot at that point.
                self.updated.insert(self.current_slot);
            }
            match event.code {
                ABS_MT_SLOT => self.current_slot = TouchPointSlot::new(event.value.max(0) as u32),
                ABS_MT_TRACKING_ID => {
                    let tracking_id = TrackingId::new(event.value);
                    if tracking_id.is_lift_off() {
                        self.slots.remove(&self.current_slot);
                    } else {
                        let entry =
                            self.slots
                                .entry(self.current_slot)
                                .or_insert_with(|| TouchPoint {
                                    slot: self.current_slot,
                                    tracking_id,
                                    pos: Vec2::new(0.0, 0.0),
                                });
                        entry.tracking_id = tracking_id;
                    }
                }
                ABS_MT_POSITION_X => {
                    if let Some(touch_point) = self.slots.get_mut(&self.current_slot) {
                        touch_point.pos.x = event.value as f64;
                        if let Some(calibration) = self.calibration {
                            touch_point.pos.x /= calibration.x_res.as_f64();
                        }
                    }
                }
                ABS_MT_POSITION_Y => {
                    if let Some(touch_point) = self.slots.get_mut(&self.current_slot) {
                        touch_point.pos.y = event.value as f64;
                        if let Some(calibration) = self.calibration {
                            touch_point.pos.y /= calibration.y_res.as_f64();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// All currently active touch points.
    pub fn active_touches(&self) -> Vec<TouchPoint> {
        self.slots.values().cloned().collect()
    }

    /// Number of currently active touch points.
    pub fn active_count(&self) -> usize {
        self.slots.len()
    }
}

impl Default for TouchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TouchState;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::Calibration;
    use tangible_objects_model::EventType;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;

    fn slot(slot: i32) -> RawEvent {
        RawEvent::new(EventType::Abs, ABS_MT_SLOT, slot)
    }

    fn tracking_id(id: i32) -> RawEvent {
        RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, id)
    }

    #[test]
    fn contact_is_added_updated_and_removed() {
        let mut state = TouchState::new();
        state.process(&slot(0));
        state.process(&tracking_id(100));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, 10));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, 20));

        assert_eq!(state.active_count(), 1);
        let touches = state.active_touches();
        assert_eq!(touches[0].tracking_id, TrackingId::new(100));

        state.process(&tracking_id(-1));
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn calibration_converts_positions_to_mm() {
        let mut state = TouchState::new();
        state.set_calibration(Calibration::new(2.0, 4.0));
        state.process(&slot(0));
        state.process(&tracking_id(1));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, 40));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, 80));

        let touches = state.active_touches();
        assert!((touches[0].pos.x - 20.0).abs() < 1e-9);
        assert!((touches[0].pos.y - 20.0).abs() < 1e-9);
    }

    #[test]
    fn multiple_slots_are_tracked_independently() {
        let mut state = TouchState::new();
        state.process(&slot(0));
        state.process(&tracking_id(1));
        state.process(&slot(1));
        state.process(&tracking_id(2));
        assert_eq!(state.active_count(), 2);

        // Removing slot 0 keeps slot 1.
        state.process(&slot(0));
        state.process(&tracking_id(-1));
        assert_eq!(state.active_count(), 1);
        assert_eq!(state.active_touches()[0].slot, TouchPointSlot::new(1));
    }

    /// SYN_DROPPED resync (kernel queue overflow): the frame after the
    /// marker is authoritative - held contacts absent from the kernel
    /// state are lifted at SYN_REPORT.
    #[test]
    fn syn_dropped_resync_lifts_contacts_absent_from_kernel_state() {
        let mut state = TouchState::new();
        // Three contacts held - one real, two dropped-state zombies.
        state.process(&slot(0));
        state.process(&tracking_id(100));
        state.process(&slot(1));
        state.process(&tracking_id(101));
        state.process(&slot(2));
        state.process(&tracking_id(102));
        assert_eq!(state.active_count(), 3);

        // Kernel truth: only slot 1 survives.
        state.process(&RawEvent::syn_dropped());
        state.process(&slot(1));
        state.process(&tracking_id(101));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, 50));
        state.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, 60));
        state.process(&RawEvent::syn_report());

        assert_eq!(state.active_count(), 1);
        let touches = state.active_touches();
        assert_eq!(touches[0].slot, TouchPointSlot::new(1));
        assert_eq!(touches[0].tracking_id, TrackingId::new(101));

        // The next frame is normal again - no wipe semantics linger.
        state.process(&RawEvent::syn_report());
        assert_eq!(state.active_count(), 1);
    }

    /// A resync batch reporting zero contacts lifts every held contact -
    /// the kernel really did lose all of them.
    #[test]
    fn syn_dropped_with_empty_kernel_state_lifts_everything() {
        let mut state = TouchState::new();
        state.process(&slot(0));
        state.process(&tracking_id(100));
        assert_eq!(state.active_count(), 1);

        state.process(&RawEvent::syn_dropped());
        state.process(&RawEvent::syn_report());
        assert_eq!(state.active_count(), 0);
    }
}
