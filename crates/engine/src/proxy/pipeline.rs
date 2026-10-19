use std::collections::HashSet;

use evdev::InputEvent;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_model::TouchSet;

use super::LegacyMode;
use super::boundary::EmitBoundary;
use super::edge_stage::EdgeStage;
use super::stage::Stage;
use super::stage_frame::ScopedMatches;
use super::stages::Stages;
use super::tangible_stage::TangibleStage;
use super::zone_stage::ZoneStage;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;
use tangible_objects_model::StageName;

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
    /// `Guarded` policy) - today's single-stage parity. Additional
    /// stages are appended with [`Self::push_stage`].
    pub fn new() -> Self {
        Self {
            stages: vec![Box::new(TangibleStage::new()) as Box<dyn Stage>].into(),
            emit: EmitBoundary::new(),
        }
    }

    /// Appends an edge stage: contacts within `margin` of the `surface`
    /// bounds are suppressed - ghost contacts some panels emit at the
    /// axis bounds.
    pub fn push_edge_stage(&mut self, margin: Millimeters, surface: Region) {
        self.stages.push(Box::new(EdgeStage::new(margin, surface)));
    }

    /// Appends a zone stage: any contact inside one of `zones` is
    /// suppressed - play-area masking.
    pub fn push_zone_stage(&mut self, zones: Vec<Region>) {
        self.stages.push(Box::new(ZoneStage::new(zones)));
    }

    /// Ensures a tangible stage exists for `scope` - created lazily the
    /// first time a `stage = "name"` profile registers (EVENT_PIPELINE.md
    /// phase 4). New stages start with empty hysteresis - the
    /// conservative default: nothing suppressed until their own gate
    /// passes.
    pub fn ensure_tangible_stage(&mut self, scope: StageName) {
        if self.stages.find_by_scope(&scope).is_none() {
            self.stages
                .push(Box::new(TangibleStage::with_scope(Some(scope))));
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
    /// phase - `SetFilterMode` reaches every stage with a `FilterMode`.
    pub fn set_mode(&mut self, mode: FilterMode, stability: Option<FrameCount>) {
        self.stages.set_mode(mode, stability);
    }

    /// Buffers one source event for the current frame.
    pub fn push(&mut self, event: RawEvent) {
        self.emit.push(event);
    }

    /// Finishes the buffered frame with `matched` treated as the general
    /// stage's set - the unscoped convenience for single-scope setups.
    pub fn finish(
        &mut self,
        matched: &HashSet<TouchPointSlot>,
        touches: TouchSet<'_>,
    ) -> Vec<InputEvent> {
        let scoped = ScopedMatches::from([(None, matched.clone())]);
        self.finish_scoped(&scoped, touches)
    }

    /// Finishes the buffered frame: every stage classifies the subset of
    /// `matched` keyed by its own scope (input masking), then the emit
    /// boundary merges the stage decisions - the mechanism sees the
    /// union - into the emitted frame, ending in `SYN_REPORT`.
    pub fn finish_scoped(
        &mut self,
        matched: &ScopedMatches,
        touches: TouchSet<'_>,
    ) -> Vec<InputEvent> {
        self.stages.classify(matched, touches);
        let union: HashSet<TouchPointSlot> = matched.values().flatten().copied().collect();
        self.emit.finish(&mut self.stages, &union)
    }
}

#[cfg(test)]
mod tests {
    use super::Pipeline;
    use crate::proxy::tangible_stage::SUPPRESSION_HYSTERESIS;
    use std::collections::HashSet;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::ABS_X;
    use tangible_objects_model::BTN_TOUCH;
    use tangible_objects_model::EventCode;
    use tangible_objects_model::EventType;
    use tangible_objects_model::FilterMode;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::Region;
    use tangible_objects_model::SYN_REPORT;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TouchSet;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

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
        pipeline.finish(&matched, TouchSet::EMPTY);
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
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
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
        let out = pipeline.finish(&HashSet::from([TouchPointSlot::new(0)]), TouchSet::EMPTY);
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
        pipeline.finish(&matched, TouchSet::EMPTY);
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_POSITION_X, 110),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_POSITION_X, 501),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
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
        pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        // Frames 2+3: the matcher decides slot 2 is a tangible twice in a
        // row - on the second frame the suppression graduates.
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 300), syn()] {
            pipeline.push(ev);
        }
        pipeline.finish(&HashSet::from([TouchPointSlot::new(2)]), TouchSet::EMPTY);
        for ev in [abs(ABS_MT_SLOT, 2), abs(ABS_MT_POSITION_X, 301), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::from([TouchPointSlot::new(2)]), TouchSet::EMPTY);
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
        pipeline.set_mode(FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // Frame without a match (flap) - the position still must not leak.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 321));
    }

    #[test]
    fn suppression_expires_without_matches() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // Hysteresis + 1 frames without a match → the position leaks again.
        for _ in 0..=SUPPRESSION_HYSTERESIS.as_u64() + 1 {
            for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 1), syn()] {
                pipeline.push(ev);
            }
            pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        }
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 321), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().any(|e| e.value() == 321));
    }

    #[test]
    fn lifted_tangible_releases_the_slot() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        suppress(&mut pipeline, 0);
        // The tangible is lifted: TRACKING_ID -1 arrives unsuppressed.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_TRACKING_ID, -1), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().any(|e| e.value() == -1));
    }

    #[test]
    fn guarded_holds_then_releases_a_finger() {
        let mut pipeline = Pipeline::new(); // Guarded is the default
        // Frame 1: new contact - withheld entirely.
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        // Frame 2: first unmatched frame - still held (gate symmetric).
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 501));
        // Frame 3: second consecutive unmatched frame reaches the gate -
        // the contact is replayed and flows.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
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
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
        assert!(out.iter().any(|e| e.value() == 321));
        // The third consecutive match graduates the slot: suppression
        // already applies to that frame's own events.
        for ev in [abs(ABS_MT_SLOT, 0), abs(ABS_MT_POSITION_X, 322), syn()] {
            pipeline.push(ev);
        }
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 322));
    }

    #[test]
    fn guarded_never_leaks_a_tangible() {
        let mut pipeline = Pipeline::new();
        let matched = HashSet::from([TouchPointSlot::new(0)]);
        // Frames 1+2: the contact matches - its events are withheld and
        // then dropped on graduation; the clone never sees the contact.
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 500 && e.value() != 7));
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 501 && e.value() != 7));
        // Frame 3: suppressed - positions are dropped too.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&matched, TouchSet::EMPTY);
        assert!(out.iter().all(|e| e.value() != 502));
    }

    #[test]
    fn tangibles_only_drops_everything_but_sync() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::TangiblesOnly, None);
        contact_frame(&mut pipeline, 0, 7, 500);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        // Only the closing SYN survives - no contact events at all.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].code(), SYN_REPORT.as_u16());
    }

    #[test]
    fn tangibles_only_retracts_previously_emitted_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        // Frame 1 in Gated: a finger contact is emitted live.
        contact_frame(&mut pipeline, 0, 7, 500);
        pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        // Switch to TangiblesOnly: the live contact is retracted once.
        pipeline.set_mode(FilterMode::TangiblesOnly, None);
        contact_frame(&mut pipeline, 0, 7, 501);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(
            out.iter()
                .any(|e| e.code() == ABS_MT_TRACKING_ID.as_u16() && e.value() == -1)
        );
        // Frame 2 in TangiblesOnly: quiet - only SYN.
        contact_frame(&mut pipeline, 0, 7, 502);
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn legacy_events_dropped_when_no_contact_is_live() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
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
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert_eq!(out.len(), 1); // SYN only
    }

    #[test]
    fn legacy_events_pass_with_a_live_contact() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
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
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().any(is_legacy_axis));
        assert!(out.iter().any(is_btn_touch));
    }

    #[test]
    fn legacy_mode_never_drops_even_with_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        pipeline.set_legacy_mode(super::LegacyMode::Never);
        contact_frame(&mut pipeline, 0, 7, 100);
        pipeline.push(abs(ABS_X, 100));
        pipeline.push(RawEvent::new(EventType::Key, BTN_TOUCH, 1));
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().all(|e| !is_legacy_axis(e)));
        assert!(out.iter().all(|e| !is_btn_touch(e)));
    }

    #[test]
    fn legacy_mode_always_emits_without_contacts() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        pipeline.set_legacy_mode(super::LegacyMode::Always);
        pipeline.push(abs(ABS_X, 77));
        pipeline.push(syn());
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().any(is_legacy_axis));
    }

    #[test]
    fn legacy_lift_emits_btn_touch_up() {
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        // Finger down: contact emitted live, BTN_TOUCH=1 passes.
        contact_frame(&mut pipeline, 0, 7, 100);
        pipeline.push(RawEvent::new(EventType::Key, BTN_TOUCH, 1));
        pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
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
        let out = pipeline.finish(&HashSet::new(), TouchSet::EMPTY);
        assert!(out.iter().any(|e| is_btn_touch(e) && e.value() == 0));
    }

    fn touch_point(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(slot as i32),
            pos: Vec2::new(x, y),
        }
    }

    #[test]
    fn edge_stage_suppresses_margin_contacts() {
        // Surface 0..100mm on both axes, 10mm margin - a contact at
        // x=5 is a ghost, one at x=50 is a finger.
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        pipeline.push_edge_stage(
            Millimeters::new(10.0),
            Region {
                x_min: Millimeters::new(0.0),
                y_min: Millimeters::new(0.0),
                x_max: Millimeters::new(100.0),
                y_max: Millimeters::new(100.0),
            },
        );
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_TRACKING_ID, 1),
            abs(ABS_MT_POSITION_X, 5),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_TRACKING_ID, 2),
            abs(ABS_MT_POSITION_X, 500),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let touches = [touch_point(0, 5.0, 50.0), touch_point(1, 50.0, 50.0)];
        let out = pipeline.finish(&HashSet::new(), (&touches[..]).into());
        // Ghost's position gone, finger's survives.
        assert!(out.iter().all(|e| e.value() != 5));
        assert!(out.iter().any(|e| e.value() == 500));
    }

    #[test]
    fn zone_stage_suppresses_masked_contacts() {
        // Zone covers x 0..200 - a contact at x=100 is masked, x=400 not.
        let mut pipeline = Pipeline::new();
        pipeline.set_mode(FilterMode::Gated, None);
        pipeline.push_zone_stage(vec![Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(200.0),
            y_max: Millimeters::new(300.0),
        }]);
        for ev in [
            abs(ABS_MT_SLOT, 0),
            abs(ABS_MT_TRACKING_ID, 1),
            abs(ABS_MT_POSITION_X, 700),
            abs(ABS_MT_SLOT, 1),
            abs(ABS_MT_TRACKING_ID, 2),
            abs(ABS_MT_POSITION_X, 900),
            syn(),
        ] {
            pipeline.push(ev);
        }
        let touches = [touch_point(0, 100.0, 50.0), touch_point(1, 400.0, 50.0)];
        let out = pipeline.finish(&HashSet::new(), (&touches[..]).into());
        assert!(out.iter().all(|e| e.value() != 700));
        assert!(out.iter().any(|e| e.value() == 900));
    }
}
