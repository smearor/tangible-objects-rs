use tangible_objects_model::ContactCount;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchSet;

use tracing::warn;

use super::CellState;
use super::EngineEvent;

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

    /// Fields currently holding a confirmed contact (debounced
    /// `stable_count > 0`) - feeds the status line's occupancy count.
    pub fn occupied_count(&self) -> usize {
        self.states
            .iter()
            .flat_map(|cells| cells.iter())
            .filter(|cell| !cell.stable_count.is_zero())
            .count()
    }

    /// Total field count across all loaded sets.
    pub fn field_count(&self) -> usize {
        self.sets.iter().map(|set| set.fields.len()).sum()
    }

    /// The loaded field sets.
    pub fn sets(&self) -> &[FieldSet] {
        &self.sets
    }

    /// Adds one field set at runtime (dynamic registration). The caller
    /// validates disjointness and name uniqueness; the set's cell states
    /// start empty.
    pub fn add_set(&mut self, set: FieldSet) {
        self.states
            .push(vec![CellState::default(); set.fields.len()]);
        self.sets.push(set);
    }

    /// Removes a field set by name. Returns `true` when removed.
    pub fn remove_set(&mut self, name: &FieldSetName) -> bool {
        let Some(index) = self.sets.iter().position(|set| set.name == *name) else {
            return false;
        };
        self.sets.remove(index);
        self.states.remove(index);
        true
    }

    /// Slots whose touches sit inside a field of a `suppress`-enabled
    /// set - merged into the proxy's suppression set so marker contacts
    /// never reach the OS (SINGLE_CONTACT_FIELD.md).
    ///
    /// Suppression is immediate, not debounced: a marker contact is
    /// withheld from the very first frame it appears inside a field.
    pub fn suppressed_slots(
        &self,
        touches: TouchSet<'_>,
    ) -> std::collections::HashSet<tangible_objects_model::TouchPointSlot> {
        touches
            .iter()
            .filter(|touch| {
                self.sets.iter().any(|set| {
                    set.suppress
                        && set
                            .fields
                            .iter()
                            .any(|field| set.field_contains(field, touch))
                })
            })
            .map(|touch| touch.slot)
            .collect()
    }

    /// Evaluates a frame's touches against all fields and returns the
    /// debounced occupancy events.
    pub fn update(&mut self, touches: TouchSet<'_>) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        for (set, cells) in self.sets.iter().zip(self.states.iter_mut()) {
            for (field, cell) in set.fields.iter().zip(cells.iter_mut()) {
                let raw = ContactCount::from(
                    touches
                        .iter()
                        .filter(|touch| set.field_contains(field, touch))
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
            name: "board".into(),
            window: None,
            stability: FrameCount::new(2),
            suppress: false,
            fields: vec![Field {
                name: "a1".into(),
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
        let events = tracker.update((&[touch(0, 50.0, 50.0)]).into());
        assert!(event_names(&events).is_empty());
        // Second consecutive frame: placed fires.
        let events = tracker.update((&[touch(0, 50.0, 50.0)]).into());
        assert!(event_names(&events).contains("placed"));
    }

    #[test]
    fn single_frame_dropout_keeps_occupancy() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        tracker.update((&[touch(0, 50.0, 50.0)]).into());
        tracker.update((&[touch(0, 50.0, 50.0)]).into()); // placed

        // One empty frame - not enough to lift.
        let events = tracker.update((&[]).into());
        assert!(!event_names(&events).contains("lifted"));
        // Second empty frame: lifted fires.
        let events = tracker.update((&[]).into());
        assert!(event_names(&events).contains("lifted"));
    }

    #[test]
    fn count_transitions_are_reported() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        let two = [touch(0, 50.0, 50.0), touch(1, 60.0, 60.0)];
        tracker.update((&two).into());
        let events = tracker.update((&two).into());
        assert!(events.iter().any(|e| matches!(
            e,
            EngineEvent::FieldCountChanged { count, .. } if count.as_u32() == 2
        )));
    }

    #[test]
    fn touches_outside_fields_are_ignored() {
        let mut tracker = FieldTracker::new();
        tracker.set_fields(vec![set()]);
        let events = tracker.update((&[touch(0, 500.0, 500.0)]).into());
        assert!(events.is_empty());
        let events = tracker.update((&[touch(0, 500.0, 500.0)]).into());
        assert!(events.is_empty());
    }
}
