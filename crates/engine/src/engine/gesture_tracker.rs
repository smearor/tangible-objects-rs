//! `GestureTracker` - accumulates per-slot trajectories and
//! dispatches completed strokes against registered spells
//! (SPELLS.md's recognition core).
//!
//! Arming (SPELLS.md phase model): a spell only matches when its
//! declared arming condition was satisfied for the episode:
//!
//! - `free` - always armed (legacy/freeform gestures).
//! - `exclusive-zone` - every stroke point must stay inside the
//!   zone's region.
//! - `start-spot` - the episode's first contact must land inside
//!   the spot's region.
//! - `five-finger-tap` - a short simultaneous five-contact tap
//!   latches arming for a window; the next episode may draw with
//!   fewer fingers.
//! - `tangible-chord` - a tracked tangible of the named pattern
//!   acts as the wand; its center path is the stroke and lifting
//!   the tangible completes the spell.

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::PatternName;
use tangible_objects_model::Region;
use tangible_objects_model::SlotSet;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellMatched;
use tangible_objects_model::SpellName;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_model::TrackedTangible;

use super::Trajectory;
use crate::engine::spell_training::SpellTraining;

/// Minimum stroke path length for a slot to count as a gesture
/// candidate (mm) - below it a contact is a rest or a normal touch
/// (SPELLS.md's temporal discriminator).
const MIN_STROKE_PATH_MM: f64 = 20.0;

/// Minimum distance between appended trajectory samples (mm) -
/// dedupes the position stream so the polyline stays sparse.
const SAMPLE_STEP_MM: f64 = 2.0;

/// Maximum spread between the five contacts' down events for a
/// five-finger tap to count as simultaneous (SPELLS.md).
const TAP_SPREAD: Duration = Duration::from_millis(150);

/// Maximum total duration of a five-finger tap (down to last lift).
const TAP_TOTAL: Duration = Duration::from_millis(400);

/// How long a completed five-finger tap keeps the arming latch.
const TAP_LATCH: Duration = Duration::from_secs(10);

/// Number of simultaneous contacts forming a five-finger tap.
const TAP_CONTACTS: usize = 5;

/// Whether an episode satisfied a spell's arming condition - the
/// `Listening -> Armed` transition's per-episode evaluation.
struct Episode<'a> {
    /// Completed strokes of the episode with their slots - the
    /// slots identify what a matched spell consumed.
    strokes: &'a [(TouchPointSlot, Vec<Vec2<Millimeters>>)],
    /// The episode's very first contact point - `start-spot` checks
    /// this against its region.
    first_point: Option<Vec2<Millimeters>>,
    /// `five-finger-tap` latch active during the episode.
    five_tap_armed: bool,
}

impl Episode<'_> {
    /// Stroke points without slot identity - the matcher input.
    fn polylines(&self) -> Vec<Vec<Vec2<Millimeters>>> {
        self.strokes
            .iter()
            .map(|(_, points)| points.clone())
            .collect()
    }
}

/// Accumulates per-slot stroke trajectories and recognizes spells
/// once a gesture episode completes.
///
/// An episode spans from the first contact until every gesture slot
/// has lifted (`Dispatch` in SPELLS.md's phase model). The tracker
/// evaluates each spell's [`SpellArming`] against the episode's
/// facts - unsatisfied arming excludes the spell from matching.
#[derive(Default)]
pub struct GestureTracker {
    /// Registered spells to match completed strokes against.
    spells: Vec<Spell>,
    /// Live stroke per slot - exists while the contact is down.
    trajectories: HashMap<TouchPointSlot, Trajectory>,
    /// Strokes completed during the current episode (slot lifted
    /// while others still move), keyed by their slot so a matched
    /// spell can name the contacts it consumed.
    completed: Vec<(TouchPointSlot, Vec<Vec2<Millimeters>>)>,
    /// The episode's first contact point - `start-spot` arming.
    episode_first: Option<Vec2<Millimeters>>,
    /// Down timestamp per live slot - five-finger-tap detection.
    down_at: HashMap<TouchPointSlot, Instant>,
    /// Candidate tap: earliest down time plus the contact group.
    pending_tap: Option<(Instant, HashSet<TouchPointSlot>)>,
    /// `five-finger-tap` latch deadline - armed while in the future.
    five_tap_armed_until: Option<Instant>,
    /// Slots consumed by the last dispatched spell - the boundary
    /// drops their withheld events instead of replaying them.
    consumed: SlotSet,
    /// Active spell-training session - episodes are captured as
    /// template repetitions instead of matched (SPELLS.md).
    training: Option<SpellTraining>,
    /// The spell the last training session derived - the device loop
    /// picks it up and registers it persistently.
    trained: Option<Spell>,
    /// Center trajectory per wand tangible, keyed by tangible ID -
    /// only tangibles whose pattern backs a `tangible-chord` spell.
    chords: HashMap<TangibleId, (PatternName, Trajectory)>,
}

impl GestureTracker {
    /// An empty tracker - no spells registered yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the spell set (load/reload path).
    pub fn set_spells(&mut self, spells: Vec<Spell>) {
        self.spells = spells;
    }

    /// Registers a spell at runtime (`Register`).
    pub fn add_spell(&mut self, spell: Spell) {
        self.spells.retain(|s| s.name != spell.name);
        self.spells.push(spell);
    }

    /// Unregisters a spell by name - `false` when unknown.
    pub fn remove_spell(&mut self, name: &SpellName) -> bool {
        let before = self.spells.len();
        self.spells.retain(|s| &s.name != name);
        self.spells.len() != before
    }

    /// The registered spells.
    pub fn spells(&self) -> &[Spell] {
        &self.spells
    }

    /// Live trajectories for the debug overlay.
    pub fn trajectories(&self) -> &HashMap<TouchPointSlot, Trajectory> {
        &self.trajectories
    }

    /// Slots to withhold from forwarding while their episode is
    /// armed (SPELLS.md's state-gated suppression). Only an armed
    /// episode suppresses - an unarmed stroke (a normal drag,
    /// pinch or scroll) must flow through untouched, so arming is
    /// evaluated on the live trajectories, not on dispatch.
    pub fn gesture_slots(&self, now: Instant) -> SlotSet {
        if !self.armed_episode_active(now) {
            return SlotSet::default();
        }
        self.trajectories.keys().copied().collect()
    }

    /// Slots consumed by a dispatched spell - their withheld events
    /// must drop permanently, never replay (a matched stroke is the
    /// input itself, not deferred desktop input).
    pub fn take_consumed(&mut self) -> SlotSet {
        std::mem::take(&mut self.consumed)
    }

    /// Whether any registered spell's arming is satisfied by the
    /// in-progress episode - the live counterpart of
    /// [`GestureTracker::episode_armed`] used for suppression.
    ///
    /// `free` and `tangible-chord` never arm a touch episode:
    /// freeform strokes flow through (SPELLS.md's defer-and-replay
    /// caveat) and chord slots are already suppressed via their
    /// tangible binding.
    fn armed_episode_active(&self, now: Instant) -> bool {
        self.spells.iter().any(|spell| match &spell.arming {
            SpellArming::Free | SpellArming::TangibleChord { .. } => false,
            SpellArming::ExclusiveZone { region } => self
                .trajectories
                .values()
                .all(|traj| traj.points.iter().all(|point| region.contains(point))),
            SpellArming::StartSpot { region } => self
                .episode_first
                .is_some_and(|point| region.contains(&point)),
            SpellArming::FiveFingerTap => self.five_tap_armed(now),
        })
    }

    /// Feeds one frame's contacts and tracked tangibles: records
    /// five-finger taps, accumulates wand trajectories, appends
    /// positions for live slots, completes lifted slots, and
    /// dispatches when an episode closes (all gesture contacts or
    /// the wand lifted). `now` is injected for testability.
    pub fn observe_frame(
        &mut self,
        touches: &[TouchPoint],
        tracked: &[TrackedTangible],
        now: Instant,
    ) -> Option<SpellMatched> {
        self.track_five_tap(touches, now);
        let chord_match = self.track_chords(tracked);

        let live: HashSet<TouchPointSlot> = touches.iter().map(|t| t.slot).collect();
        let lifted: Vec<TouchPointSlot> = self
            .trajectories
            .keys()
            .copied()
            .filter(|slot| !live.contains(slot))
            .collect();
        for slot in lifted {
            if let Some(traj) = self.trajectories.remove(&slot)
                && traj.path_length >= MIN_STROKE_PATH_MM
            {
                self.completed.push((slot, traj.points));
            }
        }

        for touch in touches {
            let traj = self.trajectories.entry(touch.slot).or_default();
            let Some(last) = traj.points.last() else {
                traj.points.push(touch.pos);
                continue;
            };
            let step = last.distance(&touch.pos).as_f64();
            if step >= SAMPLE_STEP_MM {
                traj.path_length += step;
                traj.points.push(touch.pos);
            }
        }
        if self.episode_first.is_none() {
            self.episode_first = touches.first().map(|t| t.pos);
        }

        // Dispatch: the touch episode closes when every gesture slot
        // has lifted. A chord dispatch wins over a same-frame touch
        // dispatch - wand strokes are unambiguous by construction.
        if let Some(matched) = chord_match {
            return Some(matched);
        }
        if self.trajectories.is_empty() {
            let first_point = self.episode_first.take();
            if self.completed.is_empty() {
                return None;
            }
            let strokes = std::mem::take(&mut self.completed);
            let episode = Episode {
                strokes: &strokes,
                first_point,
                five_tap_armed: self.five_tap_armed(now),
            };
            if episode.five_tap_armed {
                self.five_tap_armed_until = None;
            }
            // A training session consumes the episode instead of
            // matching it - the strokes become template repetitions
            // (SPELLS.md's learn-by-repetition).
            if self.training.is_some() {
                self.capture_training(episode.polylines());
                return None;
            }
            return self.dispatch(&episode);
        }
        None
    }

    /// Feeds a completed episode into the training buffer; a full
    /// session moves the finished spell into `trained` and disarms.
    fn capture_training(&mut self, strokes: Vec<Vec<Vec2<Millimeters>>>) {
        if let Some(training) = self.training.as_mut()
            && let Some(spell) = training.capture(strokes)
        {
            self.trained = Some(spell);
            self.training = None;
        }
    }

    /// Arms spell training (`TrainSpell` command): episodes are
    /// captured as template repetitions instead of matched until
    /// [`SPELL_TRAINING_REPETITIONS`] consistent ones land.
    pub fn start_training(&mut self, name: SpellName, arming: SpellArming) {
        self.training = Some(SpellTraining::new(name, arming));
    }

    /// Whether a training session is currently capturing.
    pub fn is_training(&self) -> bool {
        self.training.is_some()
    }

    /// The finished trained spell, once - `None` until the
    /// repetition count landed.
    pub fn take_trained_spell(&mut self) -> Option<Spell> {
        self.trained.take()
    }

    /// `true` while the five-finger-tap latch is held.
    fn five_tap_armed(&self, now: Instant) -> bool {
        self.five_tap_armed_until
            .is_some_and(|deadline| now < deadline)
    }

    /// Five-finger tap detection: five contacts down within
    /// [`TAP_SPREAD`] and fully lifted within [`TAP_TOTAL`] latches
    /// arming for [`TAP_LATCH`].
    fn track_five_tap(&mut self, touches: &[TouchPoint], now: Instant) {
        let live: HashSet<TouchPointSlot> = touches.iter().map(|t| t.slot).collect();
        self.down_at.retain(|slot, _| live.contains(slot));
        for touch in touches {
            self.down_at.entry(touch.slot).or_insert(now);
        }

        if let Some((started, group)) = &self.pending_tap {
            let all_lifted = group.iter().all(|slot| !live.contains(slot));
            let expired = now.duration_since(*started) > TAP_TOTAL;
            if all_lifted || expired {
                if all_lifted && !expired {
                    self.five_tap_armed_until = Some(now + TAP_LATCH);
                }
                self.pending_tap = None;
            }
            return;
        }

        // A fresh simultaneous five-contact cluster arms a pending tap.
        let mut recent: Vec<(Instant, TouchPointSlot)> = self
            .down_at
            .iter()
            .filter(|(_, at)| now.duration_since(**at) <= TAP_SPREAD)
            .map(|(slot, at)| (*at, *slot))
            .collect();
        if recent.len() >= TAP_CONTACTS {
            recent.sort_by_key(|(at, _)| *at);
            let earliest = recent.first().map(|(at, _)| *at).unwrap_or(now);
            let group: HashSet<TouchPointSlot> = recent
                .iter()
                .take(TAP_CONTACTS)
                .map(|(_, slot)| *slot)
                .collect();
            self.pending_tap = Some((earliest, group));
        }
    }

    /// Wand trajectories for `tangible-chord` spells: the center
    /// path of every tracked tangible whose pattern backs a chord
    /// spell. A disappearing tangible completes its spell episode.
    fn track_chords(&mut self, tracked: &[TrackedTangible]) -> Option<SpellMatched> {
        let live_ids: HashSet<TangibleId> = tracked.iter().map(|t| t.id).collect();
        let lifted: Vec<TangibleId> = self
            .chords
            .keys()
            .copied()
            .filter(|id| !live_ids.contains(id))
            .collect();
        let mut matched = None;
        for id in lifted {
            if let Some((pattern, traj)) = self.chords.remove(&id)
                && traj.path_length >= MIN_STROKE_PATH_MM
            {
                matched = self.dispatch_chord(&pattern, &traj.points).or(matched);
            }
        }

        // A tangible acts as wand when at least one chord spell
        // names its pattern - or accepts any tangible (`None`).
        let wand_for = |pattern: &PatternName| {
            self.spells.iter().any(|s| match &s.arming {
                SpellArming::TangibleChord { pattern: p } => {
                    p.as_ref().is_none_or(|p| p == pattern)
                }
                _ => false,
            })
        };
        for tangible in tracked {
            if !wand_for(&tangible.pattern_name) {
                continue;
            }
            let traj = &mut self
                .chords
                .entry(tangible.id)
                .or_insert_with(|| (tangible.pattern_name.clone(), Trajectory::default()))
                .1;
            let Some(last) = traj.points.last() else {
                traj.points.push(tangible.center);
                continue;
            };
            let step = last.distance(&tangible.center).as_f64();
            if step >= SAMPLE_STEP_MM {
                traj.path_length += step;
                traj.points.push(tangible.center);
            }
        }
        matched
    }

    /// Whether the episode satisfied a spell's arming condition -
    /// `tangible-chord` never matches touch episodes (it dispatches
    /// via [`GestureTracker::dispatch_chord`]).
    fn episode_armed(&self, spell: &Spell, episode: &Episode<'_>) -> bool {
        match &spell.arming {
            SpellArming::Free => true,
            SpellArming::ExclusiveZone { region } => {
                all_points_inside(&episode.polylines(), region)
            }
            SpellArming::StartSpot { region } => episode
                .first_point
                .is_some_and(|point| region.contains(&point)),
            SpellArming::FiveFingerTap => episode.five_tap_armed,
            SpellArming::TangibleChord { .. } => false,
        }
    }

    /// Evaluates the completed stroke set against all spells whose
    /// arming the episode satisfied; the lowest score wins when it
    /// sits inside the spell's corridor. A match marks the episode's
    /// slots consumed - the boundary drops their withheld events
    /// instead of replaying the stroke.
    fn dispatch(&mut self, episode: &Episode<'_>) -> Option<SpellMatched> {
        let polylines = episode.polylines();
        let mut best: Option<(&Spell, Millimeters)> = None;
        for spell in &self.spells {
            if !self.episode_armed(spell, episode) {
                continue;
            }
            let Some(score) = spell.try_match(&polylines) else {
                continue;
            };
            if score <= spell.corridor && best.is_none_or(|(_, best_score)| score < best_score) {
                best = Some((spell, score));
            }
        }
        let (spell, score) = best?;
        self.consumed = episode.strokes.iter().map(|(slot, _)| *slot).collect();
        Some(SpellMatched {
            name: spell.name.clone(),
            center: Trajectory::centroid(&polylines),
            score,
        })
    }

    /// Wand dispatch: the lifted tangible's center path is the sole
    /// stroke, matched against every `tangible-chord` spell naming
    /// the tangible's pattern.
    fn dispatch_chord(
        &self,
        pattern: &PatternName,
        points: &[Vec2<Millimeters>],
    ) -> Option<SpellMatched> {
        let strokes = [points.to_vec()];
        let mut best: Option<(&Spell, Millimeters)> = None;
        for spell in &self.spells {
            let armed = match &spell.arming {
                SpellArming::TangibleChord { pattern: p } => {
                    p.as_ref().is_none_or(|p| p == pattern)
                }
                _ => false,
            };
            if !armed {
                continue;
            }
            let Some(score) = spell.try_match(&strokes) else {
                continue;
            };
            if score <= spell.corridor && best.is_none_or(|(_, best_score)| score < best_score) {
                best = Some((spell, score));
            }
        }
        let (spell, score) = best?;
        Some(SpellMatched {
            name: spell.name.clone(),
            center: Trajectory::centroid(&strokes),
            score,
        })
    }
}

/// `exclusive-zone` arming: every stroke point must lie inside the
/// region - leaving the zone fails the episode.
fn all_points_inside(strokes: &[Vec<Vec2<Millimeters>>], region: &Region) -> bool {
    strokes
        .iter()
        .all(|stroke| stroke.iter().all(|point| region.contains(point)))
}

#[cfg(test)]
mod tests {
    use super::GestureTracker;
    use smearor_commons_geometry::Millimeters;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;
    use std::time::Instant;
    use tangible_objects_model::FingerCount;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternName;
    use tangible_objects_model::Region;
    use tangible_objects_model::Spell;
    use tangible_objects_model::SpellArming;
    use tangible_objects_model::SpellMatched;
    use tangible_objects_model::SpellName;
    use tangible_objects_model::StrokePolyline;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TrackedTangible;

    fn touch(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: slot.into(),
            tracking_id: (slot as i32 + 1).into(),
            pos: Vec2::new(x, y),
        }
    }

    fn v_spell(arming: SpellArming) -> Spell {
        Spell {
            name: SpellName::new("V"),
            fingers: FingerCount::new(1),
            corridor: Millimeters::new(15.0),
            scale_invariant: true,
            enabled: true,
            strokes: vec![StrokePolyline::new(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(30.0, 40.0),
                Vec2::new(60.0, 0.0),
            ])],
            arming,
        }
    }

    /// Draws the V gesture at `offset`: down-stroke then up-stroke
    /// over successive frames.
    fn draw_v(
        tracker: &mut GestureTracker,
        offset: (f64, f64),
        now: Instant,
    ) -> Option<SpellMatched> {
        let arm: Vec<(f64, f64)> = (0..=10)
            .map(|i| (offset.0 + i as f64 * 5.0, offset.1 + i as f64 * 7.0))
            .collect();
        let leg: Vec<(f64, f64)> = (0..=10)
            .map(|i| {
                (
                    offset.0 + 50.0 + i as f64 * 5.0,
                    offset.1 + 70.0 - i as f64 * 7.0,
                )
            })
            .collect();
        for (x, y) in arm.iter().chain(leg.iter()) {
            tracker.observe_frame(&[touch(0, *x, *y)], &[], now);
        }
        tracker.observe_frame(&[], &[], now)
    }

    #[test]
    fn training_derives_template_and_corridor() {
        let mut tracker = GestureTracker::new();
        tracker.start_training(SpellName::new("learned-v"), SpellArming::Free);
        let now = Instant::now();
        // Three repetitions at slight offsets - each returns None
        // (the episode is captured, not matched).
        assert!(draw_v(&mut tracker, (100.0, 100.0), now).is_none());
        assert!(tracker.is_training());
        assert!(draw_v(&mut tracker, (103.0, 98.0), now).is_none());
        assert!(draw_v(&mut tracker, (97.0, 102.0), now).is_none());
        assert!(!tracker.is_training());

        let spell = tracker
            .take_trained_spell()
            .expect("three repetitions should finish the session");
        assert_eq!(spell.name.as_str(), "learned-v");
        assert_eq!(spell.fingers.as_usize(), 1);
        assert_eq!(spell.strokes.len(), 1);
        assert!(spell.corridor.as_f64() >= 5.0);
        assert!(spell.corridor.as_f64() < 15.0);

        // The trained spell itself matches a drawn V.
        tracker.add_spell(spell);
        assert!(draw_v(&mut tracker, (200.0, 200.0), now).is_some());
    }

    #[test]
    fn free_gesture_dispatches_spell() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::Free));
        let matched = draw_v(&mut tracker, (100.0, 100.0), Instant::now()).expect("V should match");
        assert_eq!(matched.name.as_str(), "V");
        assert!(matched.score.as_f64() < 15.0);
    }

    #[test]
    fn short_contact_is_no_gesture() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::Free));
        let now = Instant::now();
        tracker.observe_frame(&[touch(0, 10.0, 10.0)], &[], now);
        tracker.observe_frame(&[touch(0, 12.0, 10.0)], &[], now);
        assert!(tracker.observe_frame(&[], &[], now).is_none());
    }

    #[test]
    fn wrong_shape_does_not_dispatch() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::Free));
        let now = Instant::now();
        for i in 0..=10 {
            tracker.observe_frame(&[touch(0, 10.0 + i as f64 * 8.0, 10.0)], &[], now);
        }
        assert!(tracker.observe_frame(&[], &[], now).is_none());
    }

    #[test]
    fn suppression_covers_only_armed_episodes() {
        let zone = Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(300.0),
            y_max: Millimeters::new(300.0),
        };
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::ExclusiveZone { region: zone }));
        let now = Instant::now();
        for i in 0..=10 {
            tracker.observe_frame(&[touch(0, 10.0 + i as f64 * 5.0, 10.0)], &[], now);
        }
        // Zone-armed episode: every episode slot is withheld.
        assert!(tracker.gesture_slots(now).contains(&0.into()));
    }

    #[test]
    fn unarmed_episodes_flow_through() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::Free));
        let now = Instant::now();
        for i in 0..=10 {
            tracker.observe_frame(&[touch(0, 10.0 + i as f64 * 5.0, 10.0)], &[], now);
        }
        // `free` never suppresses - a normal drag passes through.
        assert!(tracker.gesture_slots(now).is_empty());
    }

    #[test]
    fn exclusive_zone_arms_inside_strokes() {
        let zone = Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(300.0),
            y_max: Millimeters::new(300.0),
        };
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::ExclusiveZone { region: zone }));
        let now = Instant::now();
        // Entirely inside the zone.
        assert!(draw_v(&mut tracker, (100.0, 100.0), now).is_some());
    }

    #[test]
    fn exclusive_zone_rejects_strokes_leaving_the_zone() {
        let zone = Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(120.0),
            y_max: Millimeters::new(120.0),
        };
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::ExclusiveZone { region: zone }));
        let now = Instant::now();
        // The stroke starts inside but crosses the zone boundary.
        assert!(draw_v(&mut tracker, (100.0, 100.0), now).is_none());
    }

    #[test]
    fn start_spot_arms_only_for_strokes_beginning_inside() {
        let spot = Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(80.0),
            y_max: Millimeters::new(80.0),
        };
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::StartSpot { region: spot }));
        let now = Instant::now();
        // Starting far outside the spot never arms.
        assert!(draw_v(&mut tracker, (200.0, 200.0), now).is_none());
        // A stroke starting inside the spot arms - the rest may leave.
        assert!(draw_v(&mut tracker, (40.0, 10.0), now).is_some());
    }

    #[test]
    fn five_finger_tap_arms_the_next_episode() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::FiveFingerTap));
        let now = Instant::now();

        // The unarmed V must not match.
        assert!(draw_v(&mut tracker, (100.0, 100.0), now).is_none());

        // Tap: five contacts down together, all lifted within the
        // tap window.
        let five: Vec<TouchPoint> = (0..5)
            .map(|i| touch(i, 300.0 + i as f64 * 10.0, 300.0))
            .collect();
        tracker.observe_frame(&five, &[], now);
        tracker.observe_frame(&[], &[], now + std::time::Duration::from_millis(80));
        // Next episode armed - drawn with a single finger.
        assert!(
            draw_v(
                &mut tracker,
                (100.0, 100.0),
                now + std::time::Duration::from_millis(100),
            )
            .is_some()
        );
    }

    #[test]
    fn tangible_chord_uses_the_wand_trajectory() {
        let pattern = PatternName::new("wand");
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::TangibleChord {
            pattern: Some(pattern.clone()),
        }));
        let now = Instant::now();

        let wand = |x: f64, y: f64| TrackedTangible {
            id: TangibleId::new(7),
            pattern_id: PatternId::new(1),
            pattern_name: pattern.clone(),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::ZERO,
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: FrameCount::new(0),
            frames_since_match: FrameCount::new(0),
        };

        // Drag the wand along the V.
        let mut points: Vec<(f64, f64)> = (0..=10)
            .map(|i| (100.0 + i as f64 * 5.0, 100.0 + i as f64 * 7.0))
            .collect();
        points.extend((0..=10).map(|i| (150.0 + i as f64 * 5.0, 170.0 - i as f64 * 7.0)));
        for (x, y) in &points {
            tracker.observe_frame(&[], &[wand(*x, *y)], now);
        }
        // Lift the wand - completes the spell.
        let matched = tracker
            .observe_frame(&[], &[], now)
            .expect("wand V should match");
        assert_eq!(matched.name.as_str(), "V");
    }

    #[test]
    fn chord_spell_does_not_match_touch_episodes() {
        let mut tracker = GestureTracker::new();
        tracker.add_spell(v_spell(SpellArming::TangibleChord {
            pattern: Some(PatternName::new("wand")),
        }));
        let now = Instant::now();
        // The same V as a finger gesture is not armed.
        assert!(draw_v(&mut tracker, (100.0, 100.0), now).is_none());
    }
}
