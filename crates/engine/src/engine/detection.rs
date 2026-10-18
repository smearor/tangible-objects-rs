use std::collections::HashSet;

use tangible_objects_model::Calibration;
use tangible_objects_model::CandidateReport;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RawEvent;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tracing::debug;

use super::EngineEvent;
use super::EngineFrame;
use super::FieldTracker;
use super::TouchState;
use super::Tracker;
use super::matching;

/// Default EMA smoothing factor for tracked poses.
pub const DEFAULT_EMA_ALPHA: f64 = 0.4;

/// Default association radius between a match and a tracked tangible.
pub const DEFAULT_MAX_CENTER_DISTANCE: Millimeters = Millimeters::new(200.0);

/// Default number of frames a tangible survives without a match.
pub const DEFAULT_MAX_FRAMES_WITHOUT_MATCH: FrameCount = FrameCount::new(30);

/// Headless detection engine: slot state machine + pattern matching +
/// frame-to-frame tracking.
///
/// Feed [`RawEvent`]s via [`Engine::process`]; every `SYN_REPORT` produces an
/// [`EngineFrame`] with the frame's matches, touches and domain events, and
/// updates the tracker.
pub struct Engine {
    /// Slot state machine: accumulates `ABS_MT_*` events into active
    /// contacts between `SYN_REPORT` boundaries.
    touch_state: TouchState,
    /// Loaded figure patterns evaluated against every frame's touches.
    patterns: Vec<TangiblePattern>,
    /// Frame-to-frame tracker: nearest-neighbor association, EMA
    /// smoothing, slot bindings and expiry hysteresis.
    tracker: Tracker,
    /// Occupancy tracking for single-contact field sets.
    fields: FieldTracker,
    /// Raw-to-millimeter conversion derived from the device's `ABS_INFO`,
    /// possibly overridden by the device profile. `None` on replays
    /// without an evtest header.
    calibration: Option<Calibration>,
    /// Missing-pin completion configuration (claim radius, max missing
    /// pins, persistence, suppression mode).
    completion: CompletionConfig,
    /// Next free pattern ID for dynamic registrations - monotonic, freed
    /// IDs are never reused (DYNAMIC_PROFILES.md decision).
    next_pattern_id: PatternId,
    /// Owner (registering application) per dynamically added pattern.
    pattern_owners: std::collections::HashMap<PatternId, Option<Owner>>,
    /// Owner per dynamically added field set, keyed by set name.
    field_set_owners: std::collections::HashMap<String, Option<Owner>>,
    /// Previous frame's touches - used to emit `TouchPointsUpdated` only
    /// on actual change.
    prev_touches: Vec<TouchPoint>,
    /// Edge margin in millimeters — touches inside it are treated as ghost
    /// contacts and excluded from the frame before matching.
    edge_margin: Millimeters,
}

impl Engine {
    /// Creates an engine with the given figure patterns and default
    /// tracker parameters.
    pub fn new(patterns: Vec<TangiblePattern>) -> Self {
        let next_pattern_id = patterns
            .iter()
            .map(|pattern| pattern.id.as_u32())
            .max()
            .map_or(PatternId::new(0), |max| PatternId::new(max + 1));
        Self {
            touch_state: TouchState::new(),
            patterns,
            tracker: Tracker::new(
                DEFAULT_EMA_ALPHA,
                DEFAULT_MAX_CENTER_DISTANCE,
                DEFAULT_MAX_FRAMES_WITHOUT_MATCH,
            ),
            fields: FieldTracker::new(),
            completion: CompletionConfig::default(),
            next_pattern_id,
            pattern_owners: std::collections::HashMap::new(),
            field_set_owners: std::collections::HashMap::new(),
            calibration: None,
            prev_touches: Vec::new(),
            edge_margin: Millimeters::new(0.0),
        }
    }

    /// Sets the device calibration used for raw-to-mm conversion.
    pub fn set_calibration(&mut self, calibration: Calibration) {
        self.calibration = Some(calibration);
        self.touch_state.set_calibration(calibration);
    }

    /// Currently configured calibration, if any.
    pub fn calibration(&self) -> Option<Calibration> {
        self.calibration
    }

    /// Loads occupancy field sets (SINGLE_CONTACT_FIELD.md). Sets with
    /// overlapping fields are rejected inside the tracker.
    pub fn set_fields(&mut self, sets: Vec<FieldSet>) {
        self.fields.set_fields(sets);
    }

    /// Whether occupancy field sets are loaded.
    pub fn has_fields(&self) -> bool {
        self.fields.has_sets()
    }

    /// Sets the missing-pin completion configuration
    /// (TRACKING_STABILIZATION.md).
    pub fn set_completion(&mut self, completion: CompletionConfig) {
        self.completion = completion;
    }

    /// The active completion configuration.
    pub fn completion(&self) -> CompletionConfig {
        self.completion
    }

    /// Registers a pattern at runtime (DYNAMIC_PROFILES.md).
    ///
    /// The new pattern gets a fresh monotonic [`PatternId`]; its
    /// fingerprint is collision-checked against every existing pattern -
    /// on collision the registration is rejected and nothing changes.
    /// Returns the assigned `PatternId`.
    pub fn add_pattern(
        &mut self,
        profile: &ProfileFile,
        owner: Option<Owner>,
    ) -> Result<PatternId, String> {
        self.check_pattern_collision(profile, None)?;
        let id = self.next_pattern_id;
        self.next_pattern_id = PatternId::new(id.as_u32() + 1);
        self.pattern_owners.insert(id, owner);
        self.patterns.push(profile.to_pattern(id));
        Ok(id)
    }

    /// Replaces a registered pattern's geometry/name while keeping its
    /// ID (`UpdateProfile`). The collision check excludes the pattern
    /// itself; already-tracked tangibles keep their recorded pose - the
    /// update applies to new matches.
    pub fn update_pattern(&mut self, id: PatternId, profile: &ProfileFile) -> Result<(), String> {
        if !self.patterns.iter().any(|pattern| pattern.id == id) {
            return Err(format!("pattern {id} not found"));
        }
        self.check_pattern_collision(profile, Some(id))?;
        if let Some(pattern) = self.patterns.iter_mut().find(|pattern| pattern.id == id) {
            *pattern = profile.to_pattern(id);
        }
        Ok(())
    }

    /// Fingerprint check of `profile` against all patterns except
    /// `except` (used by add/update).
    fn check_pattern_collision(
        &self,
        profile: &ProfileFile,
        except: Option<PatternId>,
    ) -> Result<(), String> {
        let pattern = profile.to_pattern(PatternId::new(0));
        if let Some(points) = pattern.ref_points.as_deref()
            && let Some(existing) = self
                .patterns
                .iter()
                .filter(|existing| Some(existing.id) != except)
                .find(|existing| existing.matches(points))
        {
            return Err(format!(
                "fingerprint collides with registered profile '{}'",
                existing.name
            ));
        }
        Ok(())
    }

    /// Removes a pattern by ID (dynamic unregister). Its ID is never
    /// reused. Returns `true` when a pattern was removed.
    pub fn remove_pattern(&mut self, id: PatternId) -> bool {
        self.pattern_owners.remove(&id);
        let before = self.patterns.len();
        self.patterns.retain(|pattern| pattern.id != id);
        self.patterns.len() != before
    }

    /// Owner recorded for a pattern, if any.
    pub fn pattern_owner(&self, id: PatternId) -> Option<&Owner> {
        self.pattern_owners.get(&id).and_then(Option::as_ref)
    }

    /// Removes every pattern and field set owned by `owner` (disconnect
    /// cleanup). Returns `(id, name)` pairs for patterns and names for
    /// field sets - names let the caller remove persisted files too.
    pub fn remove_by_owner(&mut self, owner: &Owner) -> (Vec<(PatternId, String)>, Vec<String>) {
        let patterns: Vec<(PatternId, String)> = self
            .pattern_owners
            .iter()
            .filter(|(_, o)| o.as_ref() == Some(owner))
            .filter_map(|(id, _)| {
                self.patterns
                    .iter()
                    .find(|pattern| pattern.id == *id)
                    .map(|pattern| (*id, pattern.name.clone()))
            })
            .collect();
        for (id, _) in &patterns {
            self.remove_pattern(*id);
        }
        let set_names: Vec<String> = self
            .field_set_owners
            .iter()
            .filter(|(_, o)| o.as_ref() == Some(owner))
            .map(|(name, _)| name.clone())
            .collect();
        for name in &set_names {
            self.remove_field_set(name);
        }
        (patterns, set_names)
    }

    /// Adds an occupancy field set at runtime; rejected on overlapping
    /// fields or duplicate set name.
    pub fn add_field_set(&mut self, set: FieldSet, owner: Option<Owner>) -> Result<(), String> {
        if !set.is_disjoint() {
            return Err(format!("field set '{}' has overlapping fields", set.name));
        }
        if self
            .fields
            .sets()
            .iter()
            .any(|existing| existing.name == set.name)
        {
            return Err(format!("field set '{}' already registered", set.name));
        }
        self.field_set_owners.insert(set.name.clone(), owner);
        self.fields.add_set(set);
        Ok(())
    }

    /// Removes a field set by name. Returns `true` when removed.
    pub fn remove_field_set(&mut self, name: &str) -> bool {
        self.field_set_owners.remove(name);
        self.fields.remove_set(name)
    }

    /// All active field sets (static plus dynamically registered).
    pub fn field_sets(&self) -> &[FieldSet] {
        self.fields.sets()
    }

    /// Slots inside fields of `suppress`-enabled sets - merged into the
    /// proxy's suppression set in `--filter` mode.
    pub fn field_suppressed_slots(
        &self,
        touches: &[TouchPoint],
    ) -> std::collections::HashSet<tangible_objects_model::TouchPointSlot> {
        self.fields.suppressed_slots(touches)
    }

    /// Sets the edge margin: touches closer than `margin` to the surface
    /// bounds are dropped before matching (ghost contacts reported by some
    /// panels at `x=0`/`y=0` or the max bounds). `0` disables the filter.
    /// Needs a calibration — without mm bounds there is nothing to filter
    /// against.
    pub fn set_edge_margin(&mut self, margin: Millimeters) {
        self.edge_margin = margin;
    }

    /// The loaded figure patterns.
    pub fn patterns(&self) -> &[TangiblePattern] {
        &self.patterns
    }

    /// All currently active touch points.
    pub fn touches(&self) -> Vec<TouchPoint> {
        self.touch_state.active_touches()
    }

    /// Number of currently active touch points.
    pub fn touch_count(&self) -> usize {
        self.touch_state.active_count()
    }

    /// All currently tracked tangibles (EMA-smoothed).
    pub fn tracked(&self) -> &[TrackedTangible] {
        self.tracker.active()
    }

    /// Mutable access to the tracker (e.g. to clear it on mode switch).
    pub fn tracker_mut(&mut self) -> &mut Tracker {
        &mut self.tracker
    }

    /// Finds all pattern matches in a given set of touch points without
    /// touching engine state.
    pub fn find_matches(&self, touches: &[TouchPoint]) -> Vec<PatternMatch> {
        matching::find_matches(touches, &self.patterns)
    }

    /// Feeds one raw event into the engine.
    ///
    /// Returns `Some` with the frame's matches, touches and domain events on
    /// each `SYN_REPORT`, `None` for all other events.
    ///
    /// This is the production path: no candidate diagnostics are emitted and
    /// [`EngineFrame::candidates`] stays empty.
    pub fn process(&mut self, event: &RawEvent) -> Option<EngineFrame> {
        self.process_inner(event, None)
    }

    /// Like [`Engine::process`], additionally collecting a
    /// [`CandidateReport`] for every tested point combination - matched or
    /// rejected - into [`EngineFrame::candidates`].
    ///
    /// Used by the recording pipeline (`--record`); allocation happens only
    /// on this path.
    pub fn process_with_diagnostics(&mut self, event: &RawEvent) -> Option<EngineFrame> {
        self.process_inner(event, Some(&mut Vec::new()))
    }

    fn process_inner(
        &mut self,
        event: &RawEvent,
        mut diagnostics: Option<&mut Vec<CandidateReport>>,
    ) -> Option<EngineFrame> {
        self.touch_state.process(event);
        if !event.is_syn_report() {
            return None;
        }
        let mut touches = self.touch_state.active_touches();
        touches.sort_by_key(|touch| touch.slot);
        touches.retain(|touch| !self.is_edge_ghost(touch));

        // Slot affinity (TRACKING_STABILIZATION.md): every tracked
        // tangible re-matches on the slots it owned in the previous
        // frame, before generic matching runs.
        //
        // Pass 1 reserves *all* bound slots first - otherwise an
        // incumbent processed early could complete-claim slots that a
        // later incumbent owns (the claim pool must exclude every
        // tracked tangible's bindings, not just the ones already seen).
        let mut matches = Vec::new();
        let mut reserved: HashSet<usize> = HashSet::new();
        for tangible in self.tracker.active() {
            if let Some(slot_set) = self.tracker.slot_set(tangible.id) {
                reserved.extend(
                    touches
                        .iter()
                        .enumerate()
                        .filter(|(_, touch)| slot_set.contains(&touch.slot))
                        .map(|(index, _)| index),
                );
            }
        }

        // Pass 2: incumbent re-match on the genuinely matched slots only
        // (`match_set` excludes claimed pins - a wrong claim stays
        // reserved but can never make the incumbent fail to match).
        for tangible in self.tracker.active() {
            let Some(pattern) = self.patterns.iter().find(|p| p.id == tangible.pattern_id) else {
                continue;
            };
            let Some(match_set) = self.tracker.match_set(tangible.id) else {
                continue;
            };
            let indices: Vec<usize> = touches
                .iter()
                .enumerate()
                .filter(|(_, touch)| match_set.contains(&touch.slot))
                .map(|(index, _)| index)
                .collect();
            let affinity_match = if indices.len() == pattern.num_points {
                matching::try_pattern(pattern, &indices, &touches)
            } else if indices.len() + self.completion.max_missing as usize >= pattern.num_points
                && pattern.num_points >= 4
            {
                // Claim pool: every non-reserved touch PLUS this
                // incumbent's own bound-but-unmatched slots - a slot
                // this figure claimed last frame must stay claimable
                // for it (reserving it from others must not unclaim it
                // for its owner, otherwise the binding would oscillate).
                let own_reserved: HashSet<usize> = self
                    .tracker
                    .slot_set(tangible.id)
                    .map(|slot_set| {
                        touches
                            .iter()
                            .enumerate()
                            .filter(|(_, touch)| slot_set.contains(&touch.slot))
                            .map(|(index, _)| index)
                            .collect()
                    })
                    .unwrap_or_default();
                let pool: Vec<usize> = touches
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !reserved.contains(index) || own_reserved.contains(index))
                    .map(|(index, _)| index)
                    .collect();
                matching::try_partial(pattern, &indices, &touches, &self.completion, &pool)
            } else {
                None
            };
            if let Some(pattern_match) = affinity_match {
                matches.push(pattern_match);
            }
        }

        let generic: Vec<PatternMatch> = match diagnostics.as_mut() {
            Some(out) => matching::find_matches_with_diagnostics(
                &touches,
                &self.patterns,
                &mut **out,
                &reserved,
                &self.completion,
            ),
            None => matching::find_matches_excluding(
                &touches,
                &self.patterns,
                &reserved,
                &self.completion,
            ),
        };
        matches.extend(generic);

        let mut events = self
            .tracker
            .update(&matches, &touches, self.completion.persist);
        events.extend(self.fields.update(&touches));
        if touches != self.prev_touches {
            self.prev_touches.clone_from(&touches);
            events.push(EngineEvent::TouchPointsUpdated);
        }
        debug!(
            matches = matches.len(),
            tracked = self.tracker.active().len(),
            "frame processed"
        );
        Some(EngineFrame {
            matches,
            touches,
            events,
            candidates: diagnostics.map(std::mem::take).unwrap_or_default(),
        })
    }

    /// Whether a touch sits inside the configured edge margin — a likely
    /// ghost contact some panels emit at the axis bounds.
    ///
    /// Disabled when the margin is zero or no calibration is loaded (the mm
    /// surface bounds are unknowable then).
    fn is_edge_ghost(&self, touch: &TouchPoint) -> bool {
        let Some(cal) = &self.calibration else {
            return false;
        };
        let margin = self.edge_margin.as_f64();
        if margin <= 0.0 {
            return false;
        }
        let (x0, x1) = (
            cal.x_min.as_f64() / cal.x_res.as_f64(),
            cal.x_max.as_f64() / cal.x_res.as_f64(),
        );
        let (y0, y1) = (
            cal.y_min.as_f64() / cal.y_res.as_f64(),
            cal.y_max.as_f64() / cal.y_res.as_f64(),
        );
        touch.pos.x < x0 + margin
            || touch.pos.x > x1 - margin
            || touch.pos.y < y0 + margin
            || touch.pos.y > y1 - margin
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use crate::EngineEvent;
    use tangible_objects_model::ABS_MT_POSITION_X;
    use tangible_objects_model::ABS_MT_POSITION_Y;
    use tangible_objects_model::ABS_MT_SLOT;
    use tangible_objects_model::ABS_MT_TRACKING_ID;
    use tangible_objects_model::EventType;
    use tangible_objects_model::FieldSet;
    use tangible_objects_model::Owner;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::ProfileFile;
    use tangible_objects_model::RawEvent;
    use tangible_objects_model::SYN_REPORT;
    use tangible_objects_model::TangiblePattern;
    use tangible_objects_model::Vec2;

    fn triangle_pattern() -> TangiblePattern {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        TangiblePattern::from_points(1, "triangle", &reference, 5.0)
    }

    fn contact(engine: &mut Engine, slot: i32, id: i32, x: i32, y: i32) {
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_SLOT, slot));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, id));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, x));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, y));
    }

    fn syn(engine: &mut Engine) -> Option<crate::EngineFrame> {
        engine.process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
    }

    fn lift(engine: &mut Engine, slot: i32) {
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_SLOT, slot));
        engine.process(&RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, -1));
    }

    #[test]
    fn frame_is_emitted_on_syn_report() {
        let mut engine = Engine::new(vec![]);
        assert!(
            engine
                .process(&RawEvent::new(EventType::Abs, ABS_MT_SLOT, 0))
                .is_none()
        );
        assert!(
            engine
                .process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
                .is_some()
        );
    }

    #[test]
    fn triangle_is_detected_and_tracked() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);
        let frame = engine.process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0));

        let frame = frame.expect("frame expected on SYN_REPORT");
        assert_eq!(frame.matches.len(), 1);
        assert_eq!(frame.matches[0].pattern_name, "triangle");
        assert!(
            frame
                .events
                .iter()
                .any(|e| matches!(e, EngineEvent::TangibleAdded(_)))
        );
        assert!(
            frame
                .events
                .iter()
                .any(|e| matches!(e, EngineEvent::TouchPointsUpdated))
        );
        assert_eq!(engine.tracked().len(), 1);
    }

    #[test]
    fn process_with_diagnostics_collects_candidates() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);

        let frame = engine
            .process_with_diagnostics(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
            .expect("frame expected on SYN_REPORT");

        assert_eq!(frame.candidates.len(), 1);
        assert!(frame.candidates[0].matched);
        assert_eq!(frame.candidates[0].deviations.len(), 3);
    }

    #[test]
    fn process_leaves_candidates_empty() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 1, 0, 0);
        contact(&mut engine, 1, 2, 50, 0);
        contact(&mut engine, 2, 3, 0, 50);

        let frame = engine
            .process(&RawEvent::new(EventType::Syn, SYN_REPORT, 0))
            .expect("frame expected on SYN_REPORT");

        assert!(frame.candidates.is_empty());
    }

    fn big_triangle_pattern() -> TangiblePattern {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(80.0, 0.0),
            Vec2::new(0.0, 80.0),
        ];
        TangiblePattern::from_points(2, "big-triangle", &reference, 5.0)
    }

    #[test]
    fn bound_slots_are_reserved_for_the_incumbent() {
        // Triangle A (slots 0-2) + big triangle B (slots 3-5). After A
        // loses a pin, its remaining slots stay bound - a new touch in
        // the right spot must NOT re-form the pattern on stolen slots.
        let mut engine = Engine::new(vec![triangle_pattern(), big_triangle_pattern()]);
        contact(&mut engine, 0, 10, 100, 100);
        contact(&mut engine, 1, 11, 150, 100);
        contact(&mut engine, 2, 12, 100, 150);
        contact(&mut engine, 3, 13, 300, 300);
        contact(&mut engine, 4, 14, 380, 300);
        contact(&mut engine, 5, 15, 300, 380);
        syn(&mut engine);
        assert_eq!(engine.tracked().len(), 2);

        // A's slot 2 lifts; a new contact lands where the pin was -
        // geometrically a valid triangle with A's reserved slots.
        lift(&mut engine, 2);
        contact(&mut engine, 6, 16, 100, 150);
        let frame = syn(&mut engine);

        let frame = frame.expect("frame expected");
        // Only B may match - A's remaining slots are reserved.
        assert_eq!(frame.matches.len(), 1);
        assert_eq!(frame.matches[0].pattern_name, "big-triangle");
        let indices = &frame.matches[0].touch_indices;
        for &index in indices {
            let slot = frame.touches[index].slot;
            assert!(slot.as_u32() >= 3, "phantom stole an incumbent slot {slot}");
        }
    }

    #[test]
    fn incumbent_rematches_on_its_own_slots() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        contact(&mut engine, 0, 10, 100, 100);
        contact(&mut engine, 1, 11, 150, 100);
        contact(&mut engine, 2, 12, 100, 150);
        contact(&mut engine, 3, 20, 900, 900); // distractor
        syn(&mut engine);
        assert_eq!(engine.tracked().len(), 1);
        let id = engine.tracked()[0].id;

        // A fresh finger appears near the figure's third pin - the
        // incumbent still owns its slots and re-matches; the distractor
        // stays unmatched.
        contact(&mut engine, 4, 21, 105, 150);
        syn(&mut engine);

        assert_eq!(engine.tracked().len(), 1);
        assert_eq!(engine.tracked()[0].id, id);
    }

    #[test]
    fn add_pattern_rejects_colliding_fingerprint() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        // Same geometry as the loaded triangle - collides.
        let colliding = ProfileFile::from_points(
            "clone",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(50.0, 0.0),
                Vec2::new(0.0, 50.0),
            ],
            5.0,
        );
        assert!(engine.add_pattern(&colliding, None).is_err());

        // A different geometry registers with a fresh monotonic ID.
        let fresh = ProfileFile::from_points(
            "big",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(80.0, 0.0),
                Vec2::new(0.0, 80.0),
            ],
            5.0,
        );
        let id = engine.add_pattern(&fresh, None).expect("register pattern");
        assert_eq!(id, PatternId::new(2)); // triangle took id 1
    }

    #[test]
    fn unregister_never_reuses_pattern_ids() {
        let mut engine = Engine::new(vec![triangle_pattern()]);
        let square = ProfileFile::from_points(
            "big",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(80.0, 0.0),
                Vec2::new(0.0, 80.0),
            ],
            5.0,
        );
        let first = engine.add_pattern(&square, None).expect("register pattern");
        assert!(engine.remove_pattern(first));
        // A second registration gets a fresh ID, not the freed one.
        let second = engine
            .add_pattern(&square, None)
            .expect("register pattern again");
        assert!(second.as_u32() > first.as_u32());
    }

    #[test]
    fn update_pattern_keeps_id_and_owner() {
        let mut engine = Engine::new(vec![]);
        let small = ProfileFile::from_points(
            "pawn",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(30.0, 0.0),
                Vec2::new(0.0, 30.0),
            ],
            5.0,
        );
        let id = engine
            .add_pattern(&small, Some("chess".into()))
            .expect("register");
        let bigger = ProfileFile::from_points(
            "pawn-v2",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(80.0, 0.0),
                Vec2::new(0.0, 80.0),
            ],
            5.0,
        );
        engine.update_pattern(id, &bigger).expect("update");
        assert_eq!(engine.patterns()[0].id, id);
        assert_eq!(engine.patterns()[0].name, "pawn-v2");
        assert_eq!(engine.pattern_owner(id).map(Owner::as_str), Some("chess"));
        // Update must not collide with itself.
        assert!(engine.update_pattern(id, &bigger).is_ok());
    }

    #[test]
    fn remove_by_owner_clears_every_registration() {
        let mut engine = Engine::new(vec![]);
        let shape = ProfileFile::from_points(
            "shape",
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(30.0, 0.0),
                Vec2::new(0.0, 30.0),
            ],
            5.0,
        );
        engine
            .add_pattern(&shape, Some("game".into()))
            .expect("register");
        engine
            .add_field_set(
                FieldSet {
                    name: "grid".to_string(),
                    window: None,
                    stability: tangible_objects_model::default_stability(),
                    fields: Vec::new(),
                    suppress: false,
                },
                Some("game".into()),
            )
            .expect("register field set");
        let (patterns, sets) = engine.remove_by_owner(&"game".into());
        assert_eq!(patterns.len(), 1);
        assert_eq!(sets, vec!["grid".to_string()]);
        assert!(engine.patterns().is_empty());
        assert!(engine.field_sets().is_empty());
        // Other owners stay untouched.
        engine
            .add_pattern(&shape, Some("other".into()))
            .expect("register");
        let (patterns, _) = engine.remove_by_owner(&"game".into());
        assert!(patterns.is_empty());
        assert_eq!(engine.patterns().len(), 1);
    }

    fn quad_pattern(id: u32, name: &str) -> TangiblePattern {
        TangiblePattern::from_points(
            id,
            name,
            &[
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(0.0, 100.0),
                Vec2::new(100.0, 100.0),
            ],
            5.0,
        )
    }

    #[test]
    fn completion_never_claims_another_incumbent() {
        // A's missing-pin prediction lands within claim radius of B's
        // pin - the claim pool must still exclude B's bound slots.
        let mut engine = Engine::new(vec![quad_pattern(1, "a"), quad_pattern(2, "b")]);
        // A square at (100..200): slots 0-3.
        contact(&mut engine, 0, 10, 100, 100);
        contact(&mut engine, 1, 11, 200, 100);
        contact(&mut engine, 2, 12, 100, 200);
        contact(&mut engine, 3, 13, 200, 200);
        // B square next to it; slot 7 sits ~7mm from A's predicted
        // missing pin (200,200) - inside claim radius 10mm.
        contact(&mut engine, 4, 14, 205, 105);
        contact(&mut engine, 5, 15, 305, 105);
        contact(&mut engine, 6, 16, 205, 205);
        contact(&mut engine, 7, 17, 305, 205);
        let frame = syn(&mut engine).expect("frame");
        assert_eq!(engine.tracked().len(), 2);
        assert_eq!(frame.matches.len(), 2);

        lift(&mut engine, 3);
        let frame = syn(&mut engine).expect("frame");

        let match_a = frame
            .matches
            .iter()
            .find(|m| m.pattern_id == PatternId::new(1))
            .expect("a still partially matches");
        // Slot 6 (B's pin at 205,205) must not have been claimed by A.
        let b_pin = frame
            .touches
            .iter()
            .position(|t| t.slot.as_u32() == 6)
            .expect("b pin present");
        assert!(
            !match_a.claimed_indices.contains(&b_pin),
            "A must not claim an incumbent-owned slot"
        );
        // B still matches on all four of its slots.
        let match_b = frame
            .matches
            .iter()
            .find(|m| m.pattern_id == PatternId::new(2))
            .expect("b still matches");
        assert_eq!(match_b.all_indices().count(), 4);
    }
}
