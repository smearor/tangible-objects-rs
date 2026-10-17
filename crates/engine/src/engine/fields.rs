use tangible_objects_model::ContactCount;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::Vec2;
use tracing::warn;

use super::EngineEvent;

/// Per-field debounce state.
#[derive(Clone, Default)]
struct CellState {
    /// The last confirmed (emitted) contact count.
    stable_count: ContactCount,
    /// The candidate count currently being confirmed.
    pending_count: ContactCount,
    /// Consecutive frames at `pending_count`.
    run_length: FrameCount,
}

/// Occupancy tracker for [`FieldSet`]s.
///
/// Evaluates every frame's touches against the fields and emits
/// `FieldPlaced`/`FieldLifted`/`FieldCountChanged` events once a count
/// transition survived the set's `stability` frame budget - symmetric
/// debouncing for place and lift (SINGLE_CONTACT_FIELD.md).
pub struct FieldTracker {
    /// Loaded field sets (already disjoint-validated).
    sets: Vec<FieldSet>,
    /// Cell states parallel to `sets`/`fields`.
    states: Vec<Vec<CellState>>,
}

impl FieldTracker {
    /// Creates a tracker without any field sets.
    pub fn new() -> Self {
        Self {
            sets: Vec::new(),
            states: Vec::new(),
        }
    }

    /// Replaces the field sets; sets with overlapping fields are rejected.
    pub fn set_fields(&mut self, sets: Vec<FieldSet>) {
        let accepted: Vec<FieldSet> = sets
            .into_iter()
            .filter(|set| {
                if set.is_disjoint() {
                    true
                } else {
                    warn!(set = %set.name, "field set has overlapping fields - rejected");
                    false
                }
            })
            .collect();
        self.states = accepted
            .iter()
            .map(|set| vec![CellState::default(); set.fields.len()])
            .collect();
        self.sets = accepted;
    }

    /// Whether any field sets are loaded.
    pub fn has_sets(&self) -> bool {
        !self.sets.is_empty()
    }

    /// Evaluates a frame's touches against all fields and returns the
    /// debounced occupancy events.
    pub fn update(&mut self, touches: &[TouchPoint]) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        for (set, cells) in self.sets.iter().zip(self.states.iter_mut()) {
            for (field, cell) in set.fields.iter().zip(cells.iter_mut()) {
                let raw = ContactCount::from(
                    touches
                        .iter()
                        .filter(|touch| {
                            let local = match &set.window {
                                Some(anchor) => anchor.to_local(&touch.pos),
                                None => Vec2::new(touch.pos.x, touch.pos.y),
                            };
                            field.region.contains(&local)
                        })
                        .count(),
                );

                if raw == cell.pending_count {
                    cell.run_length += 1;
                } else {
                    cell.pending_count = raw;
                    cell.run_length = FrameCount::new(1);
                }
                if cell.run_length < set.stability || cell.pending_count == cell.stable_count {
                    continue;
                }

                let was_occupied = !cell.stable_count.is_zero();
                let is_occupied = !cell.pending_count.is_zero();
                cell.stable_count = cell.pending_count;
                if is_occupied && !was_occupied {
                    events.push(EngineEvent::FieldPlaced {
                        set: set.name.clone(),
                        field: field.name.clone(),
                    });
                } else if !is_occupied && was_occupied {
                    events.push(EngineEvent::FieldLifted {
                        set: set.name.clone(),
                        field: field.name.clone(),
                    });
                }
                events.push(EngineEvent::FieldCountChanged {
                    set: set.name.clone(),
                    field: field.name.clone(),
                    count: cell.stable_count,
                });
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::FieldTracker;
    use crate::EngineEvent;
    use tangible_objects_model::Field;
    use tangible_objects_model::FieldSet;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Region;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn touch(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(0),
            pos: Vec2::new(x, y),
        }
    }

    fn set() -> FieldSet {
        FieldSet {
            name: "board".to_string(),
            window: None,
            stability: FrameCount::new(2),
            fields: vec![Field {
                name: "a1".to_string(),
                region: Region {
                    x_min: Millimeters::new(0.0),
                    y_min: Millimeters::new(0.0),
                    x_max: Millimeters::new(100.0),
                    y_max: Millimeters::new(100.0),
                },
            }],
        }
    }

    fn event_names(events: &[EngineEvent]) -> HashSet<&'static str> {
        events
            .iter()
            .map(|e| match e {
                EngineEvent::FieldPlaced { .. } => "placed",
                EngineEvent::FieldLifted { .. } => "lifted",
                EngineEvent::FieldCountChanged { .. } => "count",
                _ => "other",
            })
            .collect()
    }

    #[test]
    fn placed_after_stability_frames() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);

        // First frame with a contact: still debouncing.
        let events = tracker.update(&[touch(0, 50.0, 50.0)]);
        assert!(event_names(&events).is_empty());
        // Second consecutive frame: placed fires.
        let events = tracker.update(&[touch(0, 50.0, 50.0)]);
        assert!(event_names(&events).contains("placed"));
    }

    #[test]
    fn single_frame_dropout_keeps_occupancy() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        tracker.update(&[touch(0, 50.0, 50.0)]);
        tracker.update(&[touch(0, 50.0, 50.0)]); // placed

        // One empty frame - not enough to lift.
        let events = tracker.update(&[]);
        assert!(!event_names(&events).contains("lifted"));
        // Second empty frame: lifted fires.
        let events = tracker.update(&[]);
        assert!(event_names(&events).contains("lifted"));
    }

    #[test]
    fn count_transitions_are_reported() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        let two = [touch(0, 50.0, 50.0), touch(1, 60.0, 60.0)];
        tracker.update(&two);
        let events = tracker.update(&two);
        assert!(events.iter().any(|e| matches!(
            e,
            EngineEvent::FieldCountChanged { count, .. } if count.as_u32() == 2
        )));
    }

    #[test]
    fn touches_outside_fields_are_ignored() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        let events = tracker.update(&[touch(0, 500.0, 500.0)]);
        assert!(events.is_empty());
        let events = tracker.update(&[touch(0, 500.0, 500.0)]);
        assert!(events.is_empty());
    }
}
