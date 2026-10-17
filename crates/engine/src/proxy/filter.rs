use std::collections::HashSet;

use evdev::InputEvent;

use tangible_objects_model::ABS_MT_SLOT;
use tangible_objects_model::ABS_MT_TRACKING_ID;
use tangible_objects_model::EventType;
use tangible_objects_model::RawEvent;
use tangible_objects_model::SYN_REPORT;

/// Frame-buffered event filter for the proxy path.
///
/// Events accumulate until `SYN_REPORT`; only then is the complete set of
/// active touch points known and the matcher can say which slots belong to a
/// tangible. Events of suppressed slots are dropped; a slot that already
/// emitted a contact before being classified gets a synthetic
/// `TRACKING_ID = -1` injection to retract it.
pub struct FilterProxy {
    /// Events of the frame currently accumulating.
    frame: Vec<RawEvent>,
    /// Source slots with a live contact on the virtual device.
    emitted: HashSet<u32>,
}

impl Default for FilterProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterProxy {
    /// Creates an empty filter proxy.
    pub fn new() -> Self {
        Self {
            frame: Vec::new(),
            emitted: HashSet::new(),
        }
    }

    /// Buffers one source event for the current frame.
    pub fn push(&mut self, event: RawEvent) {
        self.frame.push(event);
    }

    /// Finishes the buffered frame: drops events targeting `suppressed`
    /// slots and prepends `TRACKING_ID = -1` injections for suppressed slots
    /// that still have a live contact on the virtual device. Returns the
    /// complete frame to emit, ending in `SYN_REPORT`.
    pub fn finish(&mut self, suppressed: &HashSet<u32>) -> Vec<InputEvent> {
        let mut out = Vec::with_capacity(self.frame.len() + 8);

        // Retract contacts that were emitted before their slot got
        // classified as a tangible.
        let retract: Vec<u32> = suppressed
            .iter()
            .copied()
            .filter(|slot| self.emitted.contains(slot))
            .collect();
        for slot in retract {
            out.push(input_event(ABS_MT_SLOT.as_u16(), slot as i32));
            out.push(input_event(ABS_MT_TRACKING_ID.as_u16(), -1));
            self.emitted.remove(&slot);
        }

        let mut current_slot: Option<u32> = None;
        for raw in self.frame.drain(..) {
            if raw.is_syn_report() {
                continue;
            }
            if raw.event_type == EventType::Abs {
                if raw.code == ABS_MT_SLOT {
                    current_slot = (raw.value >= 0).then_some(raw.value as u32);
                }
                if current_slot.is_some_and(|slot| suppressed.contains(&slot)) {
                    continue;
                }
                if raw.code == ABS_MT_TRACKING_ID
                    && let Some(slot) = current_slot
                {
                    if raw.value == -1 {
                        self.emitted.remove(&slot);
                    } else {
                        self.emitted.insert(slot);
                    }
                }
            }
            out.push(to_input_event(&raw));
        }

        out.push(InputEvent::new(
            evdev::EventType(u16::from(EventType::Syn)),
            SYN_REPORT.as_u16(),
            0,
        ));
        out
    }
}

/// Converts a [`RawEvent`] back into an [`InputEvent`] for the clone.
///
/// The timestamp is assigned at emission; the proxy adds at most one frame
/// of latency so source timestamps are not carried over.
fn to_input_event(raw: &RawEvent) -> InputEvent {
    InputEvent::new(
        evdev::EventType(u16::from(raw.event_type)),
        raw.code.as_u16(),
        raw.value,
    )
}

/// Builds an `EV_ABS` input event for a fixed code/value pair.
fn input_event(code: u16, value: i32) -> InputEvent {
    InputEvent::new(evdev::EventType(u16::from(EventType::Abs)), code, value)
}

#[cfg(test)]
mod tests {
    use super::FilterProxy;
    use std::collections::HashSet;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::EventCode;
    use tangible_objects_model::EventType;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::SYN_REPORT;

    fn abs(code: EventCode, value: i32) -> RawEvent {
        RawEvent::new(EventType::Abs, code, value)
    }

    fn syn() -> RawEvent {
        RawEvent::new(EventType::Syn, SYN_REPORT, 0)
    }

    #[test]
    fn unsuppressed_frame_passes_through() {
        let mut proxy = FilterProxy::new();
        proxy.push(abs(ABS_MT_SLOT, 0));
        proxy.push(abs(ABS_MT_TRACKING_ID, 42));
        proxy.push(abs(ABS_MT_POSITION_X, 100));
        proxy.push(syn());
        let out = proxy.finish(&HashSet::new());
        // SLOT + TRACKING_ID + POSITION_X + SYN_REPORT
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].code(), ABS_MT_SLOT.as_u16());
        assert_eq!(out[1].value(), 42);
    }

    #[test]
    fn suppressed_slot_events_are_dropped() {
        let mut proxy = FilterProxy::new();
        // Slot 0: finger (kept), slot 1: tangible (suppressed).
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_TRACKING_ID, 10),
            abs(ABS_MT_POSITION_X, 100),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_TRACKING_ID, 11),
            abs(ABS_MT_POSITION_X, 500),
            syn(),
        ] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::from([1]));
        // Only slot 0's three events + SYN survive.
        assert_eq!(out.len(), 4);
        assert!(out.iter().all(|e| e.value() != 500));
        assert!(out.iter().all(|e| e.value() != 11));
    }

    #[test]
    fn previously_emitted_slot_gets_retracted() {
        let mut proxy = FilterProxy::new();
        // Frame 1: slot 2 emits a live contact (not yet classified).
        for ev in [
            abs(ABS_MT_SLOT, 2),
            abs(ABS_MT_TRACKING_ID, 7),
            abs(ABS_MT_POSITION_Y, 42),
            syn(),
        ] {
            proxy.push(ev);
        }
        proxy.finish(&HashSet::new());
        // Frame 2: the matcher decides slot 2 is a tangible.
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 300), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::from([2]));
        // Injection: SLOT 2 + TRACKING_ID -1, then SYN_REPORT.
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].code(), ABS_MT_SLOT.as_u16());
        assert_eq!(out[1].code(), ABS_MT_TRACKING_ID.as_u16());
        assert_eq!(out[1].value(), -1);
        // The tangible's own position event is gone.
        assert!(out.iter().all(|e| e.value() != 300));
    }

    #[test]
    fn lifted_tangible_releases_the_slot() {
        let mut proxy = FilterProxy::new();
        // Contact on slot 0, classified as tangible for one frame.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_TRACKING_ID, 3), syn()] {
            proxy.push(ev);
        }
        proxy.finish(&HashSet::from([0]));
        // The tangible is lifted: TRACKING_ID -1 arrives unsuppressed.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_TRACKING_ID, -1), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == -1));
    }
}
