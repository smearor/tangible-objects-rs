use std::collections::HashSet;

use evdev::InputEvent;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TouchPointSlot;

use super::LegacyMode;
use super::boundary::EmitBoundary;
use super::stage::TangibleStage;
use super::stages::Stages;

/// Ordered stage chain plus the shared emit boundary
/// (EVENT_PIPELINE.md phase 2).
///
/// Events accumulate until `SYN_REPORT`; per stage, [`TangibleStage`]'s
/// classification decides which slots are suppressed, then the boundary
/// shapes the emitted frame once - drops, retraction injections, guarded
/// withholding, legacy events - ending in `SYN_REPORT`.
///
/// Today the chain holds exactly one general `TangibleStage` over all
/// patterns - the backward-compatible `--filter` parity case. Later
/// phases add stage scopes, edge/zone/field stages and per-stage
/// registration.
pub struct Pipeline {
    /// Suppression stages, in declaration order.
    stages: Stages,
    /// Emit mechanism: frame buffer, emitted set, guarded withholding.
    emit: EmitBoundary,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    /// Creates a pipeline with one general tangible stage (default
    /// `Guarded` policy) - today's single-stage parity.
    pub fn new() -> Self {
        Self {
            stages: vec![TangibleStage::new()].into(),
            emit: EmitBoundary::new(),
        }
    }

    /// Switches the legacy single-touch event policy (`BTN_TOUCH`,
    /// `ABS_X`, `ABS_Y`). [`LegacyMode::Auto`] (default) emits them only
    /// while a contact is live on the virtual device.
    pub fn set_legacy_mode(&mut self, legacy: LegacyMode) {
        self.emit.set_legacy_mode(legacy);
    }

    /// Switches the suppression policy (`SetFilterMode` control command)
    /// on every stage; `stability` replaces the gate when `Some`.
    /// Per-stage control arrives with stage-scoped commands in a later
    /// phase - today one stage exists, so this is parity behavior.
    pub fn set_mode(&mut self, mode: FilterMode, stability: Option<FrameCount>) {
        self.stages.set_mode(mode, stability);
    }

    /// Buffers one source event for the current frame.
    pub fn push(&mut self, event: RawEvent) {
        self.emit.push(event);
    }

    /// Finishes the buffered frame: every stage classifies `matched` (the
    /// slots of this frame's exact matches - unscoped until stage
    /// scoping lands), then the emit boundary merges the stage decisions
    /// into the emitted frame, ending in `SYN_REPORT`.
    pub fn finish(&mut self, matched: &HashSet<TouchPointSlot>) -> Vec<InputEvent> {
        self.stages.classify(matched);
        self.emit.finish(&mut self.stages, matched)
    }
}

#[cfg(test)]
mod tests {
    use super::Pipeline;
    use std::collections::HashSet;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::ABS_X;
    use tangible_objects_model::BTN_TOUCH;
    use tangible_objects_model::EventCode;
    use tangible_objects_model::EventType;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::SYN_REPORT;
    use tangible_objects_model::TouchPointSlot;

    fn abs(code: EventCode, value: i32) -> RawEvent {
        RawEvent::new(EventType::Abs, code, value)
    }

    fn syn() -> RawEvent {
        RawEvent::syn_report()
    }

    /// True for a legacy `ABS_X` event (code 0 is shared with SYN_REPORT -
    /// the event type disambiguates).
    fn is_legacy_axis(e: &evdev::InputEvent) -> bool {
        e.event_type() == evdev::EventType::ABSOLUTE && e.code() == ABS_X.as_u16()
    }

    /// True for a `BTN_TOUCH` key event.
    fn is_btn_touch(e: &evdev::InputEvent) -> bool {
        e.event_type() == evdev::EventType::KEY && e.code() == BTN_TOUCH.as_u16()
    }

    /// Feeds a minimal frame for `slot` and finishes it with `matched`.
    fn matched_frame(pipeline: &mut Pipeline, slot: i32, matched: HashSet<TouchPointSlot>) {
        for ev in [abs(ABS_MT_SLOT, slot), abs(ABS_MT_POSITION_X, 1), syn()] {
            pipeline.push(ev);
        }
        pipeline.finish(&matched);
    }

    /// Runs `slot` through the stability gate: two consecutive matched
    /// frames, which is what suppression requires.
    fn suppress(pipeline: &mut Pipeline, slot: u32) {
        let matched = HashSet::from([TouchPointSlot::new(slot)]);
        matched_frame(pipeline, slot as i32, matched.clone());
        matched_frame(pipeline, slot as i32, matched);
    }

    /// Feeds a contact start plus one position event for `slot`.
    fn contact_frame(pipeline: &mut Pipeline, slot: i32, tid: i32, x: i32) {
        for ev in [
            abs(ABS_MT_SLOT, slot),
            abs(ABS_MT_TRACKING_ID, tid),
            abs(ABS_MT_POSITION_X, x),
            syn(),
        ] {
            pipeline.push(ev);
        }
    }

    #[test]
    fn unsuppressed_frame_passes_through() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        pipeline.push(abs(ABS_MT_SLOT, 0));
        pipeline.push(abs(ABS_MT_TRACKING_ID, 42));
        pipeline.push(abs(ABS_MT_POSITION_X, 100));
        pipeline.push(syn());
        let out = pipeline.finish(&HashSet::new());
        // SLOT + TRACKING_ID + POSITION_X + SYN_REPORT
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].code(), ABS_MT_SLOT.as_u16());
        assert_eq!(out[1].value(), 42);
    }

    #[test]
    fn single_frame_match_does_not_suppress() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // One matched frame is not enough - the slot must match in a
        // second consecutive frame before suppression starts.
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&HashSet::from([TouchPointSlot::new(0)]));
        assert!(out.iter().any(|e| e.value() == 500));
    }

    #[test]
    fn suppressed_slot_events_are_dropped() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
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
            pipeline.push(ev);
        }
        pipeline.finish(&matched);
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_POSITION_X, 110),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_POSITION_X, 501),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&matched);
        // The finger's position survives; the tangible's is dropped.
        assert!(out.iter().any(|e| e.value() == 110));
        assert!(out.iter().all(|e| e.value() != 501));
    }

    #[test]
    fn previously_emitted_slot_gets_retracted() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // Frame 1: slot 2 emits a live contact (not yet classified).
        for ev in [
            abs(ABS_MT_SLOT, 2),
            abs(ABS_MT_TRACKING_ID, 7),
            abs(ABS_MT_POSITION_Y, 42),
            syn(),
        ] {
            pipeline.push(ev);
        }
        pipeline.finish(&HashSet::new());
        // Frames 2+3: the matcher decides slot 2 is a tangible twice in a
        // row - on the second frame the suppression graduates.
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 300), syn()] {
            pipeline.push(ev);
        }
        pipeline.finish(&HashSet::from([TouchPointSlot::new(2)]));
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 301), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::from([TouchPointSlot::new(2)]));
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
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // Frame without a match (flap) - the position still must not leak.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().all(|e| e.value() != 321));
    }

    #[test]
    fn suppression_expires_without_matches() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // Hysteresis + 1 frames without a match → the position leaks again.
        for _ in 0..=crate::proxy::stage::SUPPRESSION_HYSTERESIS.as_u64() + 1 {
            for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 1), syn()] {
                pipeline.push(ev);
            }
            pipeline.finish(&HashSet::new());
        }
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == 321));
    }

    #[test]
    fn lifted_tangible_releases_the_slot() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // The tangible is lifted: TRACKING_ID -1 arrives unsuppressed.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_TRACKING_ID, -1), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == -1));
    }

    #[test]
    fn guarded_holds_then_releases_a_finger() {
        let mut pipeline = Pipeline::new(); // Guarded is the default
        // Frame 1: new contact - withheld entirely.
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        // Frame 2: first unmatched frame - still held (gate symmetric).
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 501));
        // Frame 3: second consecutive unmatched frame reaches the gate -
        // the contact is replayed and flows.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(|e| e.value() == 500));
        assert!(out.iter().any(|e| e.value() == 501));
        assert!(out.iter().any(|e| e.value() == 502));
    }

    #[test]
    fn custom_stability_requires_more_frames() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(
            tangible_objects_model::FilterMode::Gated,
            Some(tangible_objects_model::FrameCount::new(3)),
        );
        // With a gate of 3 a slot matched twice still passes through.
        let matched = HashSet::from([TouchPointSlot::new(0)]);
        matched_frame(&mut pipeline, 0, matched.clone());
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&matched);
        assert!(out.iter().any(|e| e.value() == 321));
        // The third consecutive match graduates the slot: suppression
        // already applies to that frame's own events.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 322), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 322));
    }

    #[test]
    fn guarded_never_leaks_a_tangible() {
        let mut pipeline = Pipeline::new();
        let matched = HashSet::from([TouchPointSlot::new(0)]);
        // Frames 1+2: the contact matches - its events are withheld and
        // then dropped on graduation; the clone never sees the contact.
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 501 && e.value() != 7));
        // Frame 3: suppressed - positions are dropped too.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&matched);
        assert!(out.iter().all(|e| e.value() != 502));
    }

    #[test]
    fn tangibles_only_drops_everything_but_sync() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::TangiblesOnly, None);
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&HashSet::new());
        // Only the closing SYN survives - no contact events at all.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].code(), SYN_REPORT.as_u16());
    }

    #[test]
    fn tangibles_only_retracts_previously_emitted_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // Frame 1 in Gated: a finger contact is emitted live.
        contact_frame(&mut pipeline, 0, 7, 500);
        pipeline.finish(&HashSet::new());
        // Switch to TangiblesOnly: the live contact is retracted once.
        pipeline.set_mode(tangible_objects_model::FilterMode::TangiblesOnly, None);
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&HashSet::new());
        assert!(
            out.iter()
                .any(|e| e.code() == ABS_MT_TRACKING_ID.as_u16() && e.value() == -1)
        );
        // Frame 2 in TangiblesOnly: quiet - only SYN.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&HashSet::new());
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn legacy_events_dropped_when_no_contact_is_live() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // A suppressed tangible plus legacy single-touch events: the
        // clone has no live contact, so ABS_X/BTN_TOUCH must not leak.
        suppress(&mut pipeline, 0);
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_POSITION_X, 500),
            abs(ABS_X, 500),
            RawEvent::new(EventType::Key, BTN_TOUCH, 1),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert_eq!(out.len(), 1); // SYN only
    }

    #[test]
    fn legacy_events_pass_with_a_live_contact() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // An emitted finger contact plus legacy events: Auto mirrors them.
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_TRACKING_ID, 9),
            abs(ABS_MT_POSITION_X, 100),
            abs(ABS_X, 100),
            RawEvent::new(EventType::Key, BTN_TOUCH, 1),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(is_legacy_axis));
        assert!(out.iter().any(is_btn_touch));
    }

    #[test]
    fn legacy_mode_never_drops_even_with_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        pipeline.set_legacy_mode(super::LegacyMode::Never);
        contact_frame(&mut pipeline, 0, 7, 100);
        pipeline.push(abs(ABS_X, 100));
        pipeline.push(RawEvent::new(EventType::Key, BTN_TOUCH, 1));
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().all(|e| !is_legacy_axis(e)));
        assert!(out.iter().all(|e| !is_btn_touch(e)));
    }

    #[test]
    fn legacy_mode_always_emits_without_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        pipeline.set_legacy_mode(super::LegacyMode::Always);
        pipeline.push(abs(ABS_X, 77));
        pipeline.push(syn());
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(is_legacy_axis));
    }

    #[test]
    fn legacy_lift_emits_btn_touch_up() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(tangible_objects_model::FilterMode::Gated, None);
        // Finger down: contact emitted live, BTN_TOUCH=1 passes.
        contact_frame(&mut pipeline, 0, 7, 100);
        pipeline.push(RawEvent::new(EventType::Key, BTN_TOUCH, 1));
        pipeline.finish(&HashSet::new());
        // Finger up: the frame ends with no live contact, but BTN_TOUCH=0
        // must still pass so the clone clears its key state (otherwise it
        // stays stuck at 1).
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_TRACKING_ID, -1),
            RawEvent::new(EventType::Key, BTN_TOUCH, 0),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new());
        assert!(out.iter().any(|e| is_btn_touch(e) && e.value() == 0));
    }
}
