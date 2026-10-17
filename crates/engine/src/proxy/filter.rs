use std::collections::HashMap;
use std::collections::HashSet;

use evdev::InputEvent;

use tangible_objects_model::ABS_MT_SLOT;
use tangible_objects_model::ABS_MT_TRACKING_ID;
use tangible_objects_model::EventType;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RawEvent;
use tangible_objects_model::SYN_REPORT;
use tangible_objects_model::TouchPointSlot;

/// Frames a suppressed slot survives without a renewed match - mirrors the
/// tracker's expiry hysteresis so a transient match loss (tilt, flaky pin)
/// does not briefly leak the contact to the operating system.
const SUPPRESSION_HYSTERESIS: FrameCount = FrameCount::new(30);

/// Default for [`FilterProxy::set_mode`]'s stability parameter:
/// consecutive frames a slot must be matched before suppression starts.
/// A one-frame phantom match (fingers coincidentally forming a profile)
/// never enters suppression - combined with the hysteresis, which would
/// otherwise hold the wrongly-suppressed contact for ~250 ms.
/// FILTER_PROXY_VIRTUAL.md, "Mitigation 2".
const DEFAULT_STABILITY_GATE: FrameCount = FrameCount::new(2);

/// Frame-buffered event filter for the proxy path.
///
/// Events accumulate until `SYN_REPORT`; only then is the complete set of
/// active touch points known and the matcher can say which slots belong to a
/// tangible. Events of suppressed slots are dropped; a slot that already
/// emitted a contact before being classified gets a synthetic
/// `TRACKING_ID = -1` injection to retract it.
///
/// Suppression is persistent: a slot classified as tangible stays
/// suppressed across frames without match until its tracking contact ends
/// or [`SUPPRESSION_HYSTERESIS`] expires. Entry is gated by
/// [`STABILITY_GATE`] consecutive matched frames.
pub struct FilterProxy {
    /// Events of the frame currently accumulating.
    frame: Vec<RawEvent>,
    /// Source slots with a live contact on the virtual device.
    emitted: HashSet<TouchPointSlot>,
    /// Suppressed slots mapped to frames elapsed since their last match.
    /// Resets on every renewed match; expiry releases the slot.
    suppressed: HashMap<TouchPointSlot, FrameCount>,
    /// Slots matched consecutively but not yet past the stability gate.
    /// Resets when a slot misses a frame; graduates into `suppressed`
    /// after `stability_gate` consecutive matches.
    pending: HashMap<TouchPointSlot, FrameCount>,
    /// Consecutive matched frames required before suppression starts
    /// (`Gated`/`Guarded`; `Immediate` ignores it).
    stability_gate: FrameCount,
    /// Withheld events of unclassified new contacts (`Guarded` mode):
    /// a new contact's events are buffered until the first frame
    /// evaluation - tangible contacts are dropped, finger contacts
    /// are replayed.
    held: HashMap<TouchPointSlot, Vec<RawEvent>>,
    /// Active suppression policy.
    mode: FilterMode,
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
            suppressed: HashMap::new(),
            pending: HashMap::new(),
            held: HashMap::new(),
            mode: FilterMode::default(),
            stability_gate: DEFAULT_STABILITY_GATE,
        }
    }

    /// Switches the suppression policy (`SetFilterMode` control command).
    /// `stability` replaces the gate when `Some`; `None` keeps the current
    /// value. State from the previous mode is kept: existing suppressions
    /// stay suppressed, held contacts stay held until their next
    /// evaluation.
    pub fn set_mode(&mut self, mode: FilterMode, stability: Option<FrameCount>) {
        self.mode = mode;
        if let Some(gate) = stability {
            self.stability_gate = gate;
        }
    }

    /// Buffers one source event for the current frame.
    pub fn push(&mut self, event: RawEvent) {
        self.frame.push(event);
    }

    /// Finishes the buffered frame: drops events targeting suppressed
    /// slots and prepends `TRACKING_ID = -1` injections for suppressed slots
    /// that still have a live contact on the virtual device. Returns the
    /// complete frame to emit, ending in `SYN_REPORT`.
    ///
    /// `matched` carries the slots of this frame's exact matches; they reset
    /// their hysteresis counter. Slots in earlier matches stay suppressed
    /// until their contact ends or [`SUPPRESSION_HYSTERESIS`] expires.
    pub fn finish(&mut self, matched: &HashSet<TouchPointSlot>) -> Vec<InputEvent> {
        // Classification bookkeeping per policy:
        // - `Immediate` suppresses on the first matched frame.
        // - `Gated`/`Guarded` count consecutive matches in `pending` and
        //   graduate into `suppressed` after `STABILITY_GATE`.
        // - `Off` still tracks contacts (consistent state on re-enable)
        //   but never enters suppression.
        match self.mode {
            FilterMode::Immediate => {
                for &slot in matched {
                    self.suppressed.insert(slot, FrameCount::new(0));
                }
            }
            FilterMode::Gated | FilterMode::Guarded => {
                self.pending.retain(|slot, _| matched.contains(slot));
                for &slot in matched {
                    if let std::collections::hash_map::Entry::Occupied(mut suppressed) =
                        self.suppressed.entry(slot)
                    {
                        *suppressed.get_mut() = FrameCount::new(0);
                    } else {
                        *self.pending.entry(slot).or_insert(FrameCount::new(0)) += 1;
                    }
                }
                let graduated: Vec<TouchPointSlot> = self
                    .pending
                    .iter()
                    .filter(|(_, count)| **count >= self.stability_gate)
                    .map(|(&slot, _)| slot)
                    .collect();
                for slot in graduated {
                    self.pending.remove(&slot);
                    self.suppressed.insert(slot, FrameCount::new(0));
                }
            }
            FilterMode::Off | FilterMode::TangiblesOnly => {}
        }
        // Age every suppression; expired entries are released.
        self.suppressed.retain(|_, age| {
            *age += 1;
            *age <= SUPPRESSION_HYSTERESIS
        });

        let mut out = Vec::with_capacity(self.frame.len() + 8);

        // TangiblesOnly: no contact ever reaches the clone. Slots still
        // live from an earlier policy get retracted once; classification
        // bookkeeping above stays alive for a consistent switch back.
        if self.mode == FilterMode::TangiblesOnly {
            self.frame.clear();
            self.held.clear();
            let emitted: Vec<TouchPointSlot> = self.emitted.drain().collect();
            for slot in emitted {
                out.push(input_event(ABS_MT_SLOT.as_u16(), slot.as_u32() as i32));
                out.push(input_event(ABS_MT_TRACKING_ID.as_u16(), -1));
            }
            out.push(syn_event());
            return out;
        }

        // Withheld contacts are resolved at each frame boundary. In
        // `Guarded` a held slot that matched stays held (pending tangible),
        // one that graduated is dropped, one without a match is replayed
        // as a finger contact. Every other mode drains `held` outright -
        // withheld events are replayed so nothing is lost.
        let mut release = Vec::new();
        for &slot in self.held.keys() {
            let keep = self.mode == FilterMode::Guarded
                && matched.contains(&slot)
                && !self.suppressed.contains_key(&slot);
            if !keep {
                release.push(slot);
            }
        }
        for slot in release {
            let events = self.held.remove(&slot).unwrap_or_default();
            if self.mode == FilterMode::Guarded && self.suppressed.contains_key(&slot) {
                continue; // pending tangible: drop the withheld events
            }
            for raw in events {
                if raw.event_type == EventType::Abs && raw.code == ABS_MT_TRACKING_ID {
                    if raw.value == -1 {
                        self.emitted.remove(&slot);
                    } else {
                        self.emitted.insert(slot);
                    }
                }
                out.push(to_input_event(&raw));
            }
        }

        // Retract contacts that were emitted before their slot got
        // classified as a tangible.
        let retract: Vec<TouchPointSlot> = self
            .suppressed
            .keys()
            .copied()
            .filter(|slot| self.emitted.contains(slot))
            .collect();
        for slot in retract {
            out.push(input_event(ABS_MT_SLOT.as_u16(), slot.as_u32() as i32));
            out.push(input_event(ABS_MT_TRACKING_ID.as_u16(), -1));
            self.emitted.remove(&slot);
        }

        let mut current_slot: Option<TouchPointSlot> = None;
        for raw in self.frame.drain(..) {
            if raw.is_syn_report() {
                continue;
            }
            if raw.event_type == EventType::Abs {
                if raw.code == ABS_MT_SLOT {
                    current_slot = (raw.value >= 0).then(|| TouchPointSlot::new(raw.value as u32));
                }
                // A lifting contact ends the suppression - the slot becomes
                // free for a new tracking contact. Its own -1 event is
                // still dropped; the retraction injection above already
                // closed the contact on the virtual side.
                if raw.code == ABS_MT_TRACKING_ID
                    && raw.value == -1
                    && let Some(slot) = current_slot
                {
                    self.suppressed.remove(&slot);
                }
                if self.mode != FilterMode::Off
                    && current_slot.is_some_and(|slot| self.suppressed.contains_key(&slot))
                {
                    continue;
                }
                if self.mode == FilterMode::Guarded {
                    // The slot this event applies to: the SLOT selector
                    // event names its own target, every other event
                    // belongs to the current slot.
                    let target = if raw.code == ABS_MT_SLOT {
                        (raw.value >= 0).then(|| TouchPointSlot::new(raw.value as u32))
                    } else {
                        current_slot
                    };
                    if let Some(slot) = target {
                        // A fresh contact (new TRACKING_ID on a slot that
                        // is neither live on the clone nor suppressed) is
                        // withheld until classified; events of an already
                        // held slot keep accumulating.
                        let starts_contact = raw.code == ABS_MT_TRACKING_ID
                            && raw.value >= 0
                            && !self.emitted.contains(&slot);
                        if self.held.contains_key(&slot) || starts_contact {
                            self.held.entry(slot).or_default().push(raw);
                            continue;
                        }
                    }
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

        out.push(syn_event());
        out
    }
}

/// Builds the `SYN_REPORT` event closing an emitted frame.
fn syn_event() -> InputEvent {
    InputEvent::new(
        evdev::EventType(u16::from(EventType::Syn)),
        SYN_REPORT.as_u16(),
        0,
    )
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
    use tangible_objects_model::TouchPointSlot;

    fn abs(code: EventCode, value: i32) -> RawEvent {
        RawEvent::new(EventType::Abs, code, value)
    }

    fn syn() -> RawEvent {
        RawEvent::new(EventType::Syn, SYN_REPORT, 0)
    }

    /// Feeds a minimal frame for `slot` and finishes it with `matched`.
    fn matched_frame(proxy: &mut FilterProxy, slot: i32, matched: HashSet<TouchPointSlot>) {
        for ev in [abs(ABS_MT_SLOT, slot), abs(ABS_MT_POSITION_X, 1), syn()] {
            proxy.push(ev);
        }
        proxy.finish(&matched);
    }

    /// Runs `slot` through the stability gate: two consecutive matched
    /// frames, which is what suppression requires.
    fn suppress(proxy: &mut FilterProxy, slot: u32) {
        let matched = HashSet::from([TouchPointSlot::new(slot)]);
        matched_frame(proxy, slot as i32, matched.clone());
        matched_frame(proxy, slot as i32, matched);
    }

    /// Feeds a contact start plus one position event for `slot`.
    fn contact_frame(proxy: &mut FilterProxy, slot: i32, tid: i32, x: i32) {
        for ev in [
            abs(ABS_MT_SLOT, slot),
            abs(ABS_MT_TRACKING_ID, tid),
            abs(ABS_MT_POSITION_X, x),
            syn(),
        ] {
            proxy.push(ev);
        }
    }

    #[test]
    fn unsuppressed_frame_passes_through() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
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
    fn single_frame_match_does_not_suppress() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // One matched frame is not enough - the slot must match in a
        // second consecutive frame before suppression starts.
        contact_frame(&mut proxy, 0, 7, 500);
        let out = proxy.finish(&HashSet::from([TouchPointSlot::new(0)]));
        assert!(out.iter().any(|e| e.value() == 500));
    }

    #[test]
    fn suppressed_slot_events_are_dropped() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // Slot 0: finger (kept), slot 1: tangible - suppressed after the
        // second consecutive matched frame.
        let matched = HashSet::from([TouchPointSlot::new(1)]);
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
        proxy.finish(&matched);
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_POSITION_X, 110),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_POSITION_X, 501),
            syn(),
        ] {
            proxy.push(ev);
        }
        let out = proxy.finish(&matched);
        // The finger's position survives; the tangible's is dropped.
        assert!(out.iter().any(|e| e.value() == 110));
        assert!(out.iter().all(|e| e.value() != 501));
    }

    #[test]
    fn previously_emitted_slot_gets_retracted() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
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
        // Frames 2+3: the matcher decides slot 2 is a tangible twice in a
        // row - on the second frame the suppression graduates.
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 300), syn()] {
            proxy.push(ev);
        }
        proxy.finish(&HashSet::from([TouchPointSlot::new(2)]));
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 301), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::from([TouchPointSlot::new(2)]));
        // Injection: SLOT 2 + TRACKING_ID -1, then SYN_REPORT.
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].code(), ABS_MT_SLOT.as_u16());
        assert_eq!(out[1].code(), ABS_MT_TRACKING_ID.as_u16());
        assert_eq!(out[1].value(), -1);
        // The tangible's own position event is gone.
        assert!(out.iter().all(|e| e.value() != 301));
    }

    #[test]
    fn suppression_survives_a_transient_match_loss() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut proxy, 0);
        // Frame without a match (flap) - the position still must not leak.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().all(|e| e.value() != 321));
    }

    #[test]
    fn suppression_expires_without_matches() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut proxy, 0);
        // Hysteresis + 1 frames without a match → the position leaks again.
        for _ in 0..=super::SUPPRESSION_HYSTERESIS.as_u64() + 1 {
            for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 1), syn()] {
                proxy.push(ev);
            }
            proxy.finish(&HashSet::new());
        }
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == 321));
    }

    #[test]
    fn lifted_tangible_releases_the_slot() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut proxy, 0);
        // The tangible is lifted: TRACKING_ID -1 arrives unsuppressed.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_TRACKING_ID, -1), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == -1));
    }

    #[test]
    fn guarded_holds_then_releases_a_finger() {
        let mut proxy = FilterProxy::new(); // Guarded is the default
        // Frame 1: new contact - withheld entirely.
        contact_frame(&mut proxy, 0, 7, 500);
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        // Frame 2: not matched - the contact is replayed and flows.
        contact_frame(&mut proxy, 0, 7, 501);
        let out = proxy.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == 500));
        assert!(out.iter().any(|e| e.value() == 501));
    }

    #[test]
    fn custom_stability_requires_more_frames() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(
            tangible_objects_model::FilterMode::Gated,
            Some(tangible_objects_model::FrameCount::new(3)),
        );
        // With a gate of 3 a slot matched twice still passes through.
        let matched = HashSet::from([TouchPointSlot::new(0)]);
        matched_frame(&mut proxy, 0, matched.clone());
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&matched);
        assert!(out.iter().any(|e| e.value() == 321));
        // The third consecutive match graduates the slot: suppression
        // already applies to that frame's own events.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 322), syn()] {
            proxy.push(ev);
        }
        let out = proxy.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 322));
    }

    #[test]
    fn guarded_never_leaks_a_tangible() {
        let mut proxy = FilterProxy::new();
        let matched = HashSet::from([TouchPointSlot::new(0)]);
        // Frames 1+2: the contact matches - its events are withheld and
        // then dropped on graduation; the clone never sees the contact.
        contact_frame(&mut proxy, 0, 7, 500);
        let out = proxy.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        contact_frame(&mut proxy, 0, 7, 501);
        let out = proxy.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 501 && e.value() != 7));
        // Frame 3: suppressed - positions are dropped too.
        contact_frame(&mut proxy, 0, 7, 502);
        let out = proxy.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 502));
    }

    #[test]
    fn tangibles_only_drops_everything_but_sync() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::TangiblesOnly, None);
        contact_frame(&mut proxy, 0, 7, 500);
        let out = proxy.finish(&HashSet::new());
        // Only the closing SYN survives - no contact events at all.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].code(), SYN_REPORT.as_u16());
    }

    #[test]
    fn tangibles_only_retracts_previously_emitted_contacts() {
        let mut proxy = FilterProxy::new();
        proxy.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // Frame 1 in Gated: a finger contact is emitted live.
        contact_frame(&mut proxy, 0, 7, 500);
        proxy.finish(&HashSet::new());
        // Switch to TangiblesOnly: the live contact is retracted once.
        proxy.set_mode(tangible_objects_model::FilterMode::TangiblesOnly, None);
        contact_frame(&mut proxy, 0, 7, 501);
        let out = proxy.finish(&HashSet::new());
        assert!(
            out.iter()
                .any(|e| e.code() == ABS_MT_TRACKING_ID.as_u16() && e.value() == -1)
        );
        // Frame 2 in TangiblesOnly: quiet - only SYN.
        contact_frame(&mut proxy, 0, 7, 502);
        let out = proxy.finish(&HashSet::new());
        assert_eq!(out.len(), 1);
    }
}
