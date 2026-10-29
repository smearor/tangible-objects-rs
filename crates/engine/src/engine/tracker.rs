use std::collections::HashMap;

use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::RotationRad;
use tangible_objects_model::SmoothingAlpha;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TouchSet;

use tangible_objects_model::SlotSet;
use tangible_objects_model::TrackedBuf;
use tangible_objects_model::TrackedSet;
use tangible_objects_model::TrackedTangible;
use tracing::debug;

use super::EngineEvent;
use super::pin_maps::PinMaps;

/// Frame-to-frame tracker using nearest-neighbor association and EMA
/// smoothing.
///
/// Associates each frame's [`PatternMatch`]es with already tracked tangibles
/// (same pattern ID, bounded center distance), smooths pose with an
/// exponential moving average and expires tangibles after a configurable
/// number of frames without a match.
///
/// Additionally keeps every tangible's *slot binding*: the
/// [`TouchPointSlot`]s of its last successful match. These bindings feed
/// the slot-affinity matching (TRACKING_STABILIZATION.md) - a bound slot
/// is reserved for its owner and cannot be stolen by phantom candidates.
pub struct Tracker {
    /// Currently alive tangibles with EMA-smoothed pose and
    /// `frames_since_match` age.
    tracked: TrackedBuf,
    /// Slot bindings per tangible ID - the slots of its last match,
    /// including persistent completion claims (the *reservation* set).
    slots: HashMap<TangibleId, SlotSet>,
    /// Only the genuinely matched slots (no claimed pins) - the
    /// candidate set the incumbent re-match is verified against. A
    /// wrongly claimed contact is reserved but can never make a
    /// re-match fail.
    matched_slots: HashMap<TangibleId, SlotSet>,
    /// Persistent slot→pin binding per tangible (ROTATION.md) -
    /// solved once at spawn, reused for rotation on every incumbent
    /// frame instead of the argmax `rotation_of` estimate.
    pin_map: PinMaps,
    /// Smoothing factor of the exponential moving average applied to
    /// center and rotation on every associated match (higher =
    /// faster, more jitter).
    ema_alpha: SmoothingAlpha,
    /// Maximum distance between a match's center and a tracked
    /// tangible's center for nearest-neighbor association.
    max_center_distance: Millimeters,
    /// Frames a tracked tangible survives without a renewed match
    /// (expiry hysteresis, e.g. against match flapping).
    max_frames_without_match: FrameCount,
    /// Next free [`TangibleId`] - incremented for every new tangible.
    next_tangible_id: TangibleId,
}

impl Tracker {
    /// Creates a tracker.
    ///
    /// `ema_alpha` is the smoothing factor, `max_center_distance` the
    /// association radius, `max_frames_without_match` the expiry hysteresis in
    /// frames.
    pub fn new(
        ema_alpha: SmoothingAlpha,
        max_center_distance: impl Into<Millimeters>,
        max_frames_without_match: impl Into<FrameCount>,
    ) -> Self {
        Self {
            tracked: TrackedBuf::default(),
            slots: HashMap::new(),
            matched_slots: HashMap::new(),
            pin_map: PinMaps::default(),
            ema_alpha,
            max_center_distance: max_center_distance.into(),
            max_frames_without_match: max_frames_without_match.into(),
            next_tangible_id: TangibleId::new(0),
        }
    }

    /// The slots bound to a tangible by its last match, if any
    /// (reservation set - includes persistent claims).
    pub fn slot_set(&self, id: TangibleId) -> Option<&SlotSet> {
        self.slots.get(&id)
    }

    /// Only the genuinely matched slots of the last match - the
    /// candidate set for incumbent re-matching. Claimed pins are
    /// reserved but not required to re-match.
    pub fn match_set(&self, id: TangibleId) -> Option<&SlotSet> {
        self.matched_slots.get(&id)
    }

    /// All slots currently bound to any tracked tangible - the reserved
    /// set excluded from generic matching.
    pub fn bound_slots(&self) -> SlotSet {
        self.slots.values().flatten().copied().collect()
    }

    /// Processes the matches of a new frame: associate, update, create,
    /// expire. `touches` resolves `touch_indices` to slots so every
    /// tracked tangible keeps its slot binding. `bind_claimed` controls
    /// whether completion-claimed indices join the binding
    /// (`CompletionConfig::persist`). Returns the domain events produced
    /// by this update (`TangibleMoved`/`TangibleAdded`/`TangibleRemoved`).
    pub fn update(
        &mut self,
        matches: &[PatternMatch],
        touches: TouchSet<'_>,
        bind_claimed: bool,
    ) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        for tangible in self.tracked.0.iter_mut() {
            tangible.frames_since_match += 1;
        }

        let mut used_matches = vec![false; matches.len()];
        let mut used_tracked = vec![false; self.tracked.len()];

        // Greedy nearest-neighbor association (same pattern_id only).
        for (match_index, pattern_match) in matches.iter().enumerate() {
            let mut best_ti: Option<usize> = None;
            let mut best_dist = self.max_center_distance;
            for (ti, tangible) in self.tracked.iter().enumerate() {
                if used_tracked[ti] || tangible.pattern_id != pattern_match.pattern_id {
                    continue;
                }
                let dist = tangible.center.distance(&pattern_match.center);
                if dist < best_dist {
                    best_dist = dist;
                    best_ti = Some(ti);
                }
            }
            if let Some(ti) = best_ti {
                used_matches[match_index] = true;
                used_tracked[ti] = true;
                let tangible = &mut self.tracked[ti];
                tangible.smooth_center(self.ema_alpha, pattern_match.center);
                // Rotation comes from the persisted slot→pin map
                // when one exists (deterministic, argmax-proof) -
                // the match's own angle is the fallback otherwise.
                let rotation = self
                    .pin_map
                    .rotation_of(tangible.id, touches)
                    .unwrap_or(pattern_match.rotation_rad);
                tangible.rotation_rad =
                    RotationRad::ema(tangible.rotation_rad, rotation, self.ema_alpha);
                tangible.frames_since_match = FrameCount::new(0);
                let genuine_slots = pattern_match.matched_slots(touches);
                self.matched_slots
                    .insert(tangible.id, genuine_slots.clone());
                self.slots
                    .insert(tangible.id, pattern_match.slots_of(touches, bind_claimed));
                // Map refresh: the genuine slot set changed (mass slot
                // churn) → re-bind from the fresh solved match; claims
                // always follow their current pairs.
                self.pin_map.sync(tangible.id, pattern_match, touches);
                if tangible.pending {
                    // Confirmation: an exact match on the same slot set
                    // confirms immediately; a held partial match counts
                    // toward the pattern's spawn-stability window.
                    if !pattern_match.partial {
                        tangible.pending = false;
                    } else {
                        tangible.pending_frames += 1;
                        if tangible.pending_frames >= tangible.spawn_stability {
                            tangible.pending = false;
                        }
                    }
                    if !tangible.pending {
                        events.push(EngineEvent::TangibleAdded(tangible.clone()));
                    }
                } else {
                    events.push(EngineEvent::TangibleMoved(tangible.clone()));
                }
            }
        }

        // Create new tracked tangibles for unmatched matches. A
        // partial-spawned tangible starts pending and emits no event
        // until confirmed (spawn_stability frames or an exact match).
        for (match_index, pattern_match) in matches.iter().enumerate() {
            if !used_matches[match_index] {
                let pending = pattern_match.partial;
                let tangible = TrackedTangible {
                    id: self.next_tangible_id,
                    pattern_id: pattern_match.pattern_id,
                    pattern_name: pattern_match.pattern_name.clone(),
                    center: pattern_match.center,
                    rotation_rad: pattern_match.rotation_rad,
                    pending,
                    // The spawn frame itself is the first held frame.
                    pending_frames: if pending {
                        FrameCount::new(1)
                    } else {
                        FrameCount::new(0)
                    },
                    spawn_stability: pattern_match.spawn_stability,
                    frames_since_match: FrameCount::new(0),
                };
                self.next_tangible_id = TangibleId::new(self.next_tangible_id.as_u32() + 1);
                self.matched_slots
                    .insert(tangible.id, pattern_match.matched_slots(touches));
                self.slots
                    .insert(tangible.id, pattern_match.slots_of(touches, bind_claimed));
                self.pin_map.sync(tangible.id, pattern_match, touches);
                debug!(pattern = %tangible.pattern_name, id = %tangible.id, "tangible added");
                self.tracked.push(tangible.clone());
                if !pending {
                    events.push(EngineEvent::TangibleAdded(tangible));
                }
            }
        }

        // A missed frame resets the pending confirmation counter - the
        // partial spawn must hold for consecutive frames.
        for tangible in self.tracked.0.iter_mut() {
            if tangible.pending && tangible.frames_since_match > FrameCount::new(0) {
                tangible.pending_frames = FrameCount::new(0);
            }
        }

        // Remove stale tracked tangibles - their slot bindings die with
        // them, freeing the slots for new matches.
        let (kept, removed): (Vec<_>, Vec<_>) = self
            .tracked
            .drain(..)
            .partition(|t| t.frames_since_match <= self.max_frames_without_match);
        self.tracked = kept.into();
        for tangible in removed {
            debug!(pattern = %tangible.pattern_name, "tangible removed");
            self.slots.remove(&tangible.id);
            self.matched_slots.remove(&tangible.id);
            self.pin_map.remove(&tangible.id);
            events.push(EngineEvent::TangibleRemoved(tangible));
        }
        events
    }

    /// All currently tracked tangibles.
    /// Whether any tracked tangible is a pending spawn - a partial
    /// match still collecting its `spawn_stability` frames. Gates the
    /// generic matching pass (SKIP_UNCHANGED.md): a pending spawn must
    /// keep matching until it resolves.
    pub fn has_pending(&self) -> bool {
        self.active().iter().any(|t| t.pending)
    }

    pub fn active(&self) -> TrackedSet<'_> {
        (&self.tracked).into()
    }

    /// Drops all tracked tangibles.
    pub fn clear(&mut self) {
        self.tracked.clear();
        self.slots.clear();
        self.matched_slots.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineEvent;
    use tangible_objects_model::DEFAULT_SPAWN_STABILITY;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternMatch;
    use tangible_objects_model::RefIndices;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::SmoothingAlpha;
    use tangible_objects_model::Vec2;

    fn pattern_match(id: u32, x: f64, y: f64) -> PatternMatch {
        PatternMatch {
            pattern_id: PatternId::new(id),
            pattern_name: format!("fig-{id}").into(),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: Default::default(),
            correspondence: RefIndices::default(),
            ref_points: None,
            partial: false,
            claimed_indices: Default::default(),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
        }
    }

    fn touches() -> Vec<tangible_objects_model::TouchPoint> {
        Vec::new()
    }

    #[test]
    fn new_match_creates_tracked_tangible() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(30));
        let events = tracker.update(&[pattern_match(1, 10.0, 20.0)], (&touches()).into(), true);
        assert_eq!(tracker.active().len(), 1);
        assert_eq!(tracker.active()[0].pattern_id, PatternId::new(1));
        assert!(matches!(events.as_slice(), [EngineEvent::TangibleAdded(_)]));
    }

    #[test]
    fn nearby_match_associates_and_smooths() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.5), 200.0, FrameCount::new(30));
        tracker.update(&[pattern_match(1, 10.0, 20.0)], (&touches()).into(), true);
        tracker.update(&[pattern_match(1, 20.0, 20.0)], (&touches()).into(), true);
        // EMA(0.5) of 10 and 20 → 15.
        assert!((tracker.active()[0].center.x - 15.0).abs() < 1e-9);
    }

    #[test]
    fn distant_match_creates_second_tangible() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(30));
        tracker.update(&[pattern_match(1, 10.0, 20.0)], (&touches()).into(), true);
        tracker.update(&[pattern_match(1, 500.0, 500.0)], (&touches()).into(), true);
        assert_eq!(tracker.active().len(), 2);
    }

    #[test]
    fn tangible_expires_after_max_frames() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(2));
        tracker.update(&[pattern_match(1, 10.0, 20.0)], (&touches()).into(), true);
        tracker.update(&[], (&touches()).into(), true);
        tracker.update(&[], (&touches()).into(), true);
        let events = tracker.update(&[], (&touches()).into(), true);
        assert!(tracker.active().is_empty());
        assert!(matches!(
            events.as_slice(),
            [EngineEvent::TangibleRemoved(_)]
        ));
    }

    #[test]
    fn partial_spawn_is_pending_until_spawn_stability() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(30));
        let mut partial = pattern_match(1, 10.0, 20.0);
        partial.partial = true;
        partial.spawn_stability = FrameCount::new(3);

        // A partial spawn emits no TangibleAdded - it stays pending.
        let events = tracker.update(&[partial.clone()], (&touches()).into(), true);
        assert!(events.is_empty());
        assert!(tracker.active()[0].pending);

        // Held partial matches count toward the window.
        let events = tracker.update(&[partial.clone()], (&touches()).into(), true);
        assert!(events.is_empty());
        let events = tracker.update(&[partial.clone()], (&touches()).into(), true);
        assert!(matches!(
            events.as_slice(),
            [EngineEvent::TangibleAdded(t)] if !t.pending
        ));
        // After confirmation the tangible emits TangibleMoved again.
        let events = tracker.update(&[partial.clone()], (&touches()).into(), true);
        assert!(matches!(events.as_slice(), [EngineEvent::TangibleMoved(_)]));
    }

    #[test]
    fn pending_exact_match_confirms_immediately() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(30));
        let mut partial = pattern_match(1, 10.0, 20.0);
        partial.partial = true;
        partial.spawn_stability = DEFAULT_SPAWN_STABILITY;
        tracker.update(&[partial], (&touches()).into(), true);
        assert!(tracker.active()[0].pending);

        // An exact match on the same slot set confirms instantly.
        let events = tracker.update(&[pattern_match(1, 10.0, 20.0)], (&touches()).into(), true);
        assert!(matches!(
            events.as_slice(),
            [EngineEvent::TangibleAdded(t)] if !t.pending
        ));
    }

    #[test]
    fn pending_counter_resets_on_missed_frame() {
        let mut tracker = Tracker::new(SmoothingAlpha::new(0.4), 200.0, FrameCount::new(30));
        let mut partial = pattern_match(1, 10.0, 20.0);
        partial.partial = true;
        partial.spawn_stability = FrameCount::new(2);
        tracker.update(&[partial.clone()], (&touches()).into(), true);
        tracker.update(&[], (&touches()).into(), true);
        // The miss reset the counter: one more held frame does not
        // confirm yet.
        let events = tracker.update(&[partial.clone()], (&touches()).into(), true);
        assert!(events.is_empty());
        assert!(tracker.active()[0].pending);
    }

    #[test]
    fn ema_angle_wraps_around() {
        // Crossing the -PI/+PI boundary takes the short path: 3.0 -> ~PI,
        // not down through 0.
        let smoothed = RotationRad::ema(
            RotationRad::new(3.0),
            RotationRad::new(-3.0),
            SmoothingAlpha::new(0.5),
        )
        .as_f64();
        assert!(smoothed > 3.0);
        assert!(smoothed <= std::f64::consts::PI + 1e-9);
    }
}
