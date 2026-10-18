use std::collections::HashMap;
use std::collections::HashSet;

use super::SetAction;
use super::TangibleSet;
use super::TangibleSetId;
use crate::TangibleId;
use crate::TrackedTangible;

/// Per-set runtime state of the [`SetTracker`].
struct SetState {
    /// Whether the set is currently active (all mandatory members were seen).
    active: bool,
    /// Member tangibles known from the previous update.
    members: HashMap<TangibleId, TrackedTangible>,
}

/// Derives Tangible Object Set actions from the stream of tracked tangibles.
///
/// Fed with the engine's [`TrackedTangible`]s (e.g. from `TangibleAdded`,
/// `TangibleMoved`, `TangibleRemoved` events). Implements the activation
/// hysteresis: a set activates when **all** mandatory members are present
/// and deactivates only when **no** mandatory member remains.
pub struct SetTracker {
    sets: Vec<TangibleSet>,
    states: HashMap<TangibleSetId, SetState>,
}

impl SetTracker {
    /// Creates a tracker for the given set definitions.
    pub fn new(sets: Vec<TangibleSet>) -> Self {
        let states = sets
            .iter()
            .map(|set| {
                (
                    set.id,
                    SetState {
                        active: false,
                        members: HashMap::new(),
                    },
                )
            })
            .collect();
        Self { sets, states }
    }

    /// IDs of all currently active sets.
    pub fn active_set_ids(&self) -> Vec<TangibleSetId> {
        self.states
            .iter()
            .filter(|(_, state)| state.active)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Feeds the current set of tracked tangibles and returns the set
    /// actions that occurred since the previous update.
    ///
    /// Call once per frame with the full list of tracked tangibles.
    pub fn update(&mut self, tangibles: &[TrackedTangible]) -> Vec<SetAction> {
        let mut actions = Vec::new();
        for set in &self.sets {
            let Some(state) = self.states.get_mut(&set.id) else {
                continue;
            };
            let present: Vec<&TrackedTangible> = tangibles
                .iter()
                .filter(|t| set.contains(t.pattern_id))
                .collect();
            let present_ids: HashSet<TangibleId> = present.iter().map(|t| t.id).collect();
            let mandatory_present = present
                .iter()
                .filter(|t| set.is_mandatory(t.pattern_id))
                .count();
            let all_mandatory_present = set
                .mandatory
                .iter()
                .all(|p| present.iter().any(|t| t.pattern_id == *p));

            if !state.active && all_mandatory_present && !set.mandatory.is_empty() {
                state.active = true;
                actions.push(SetAction::SetAdded(set.id));
            } else if state.active && mandatory_present == 0 {
                state.active = false;
                actions.push(SetAction::SetRemoved(set.id));
                // Report every known member as removed so subscribers can
                // tear down per-object state on deactivation.
                for (_, tangible) in state.members.drain() {
                    actions.push(SetAction::ObjectRemoved(set.id, tangible));
                }
            }

            if state.active {
                // Removed members.
                let removed: Vec<TangibleId> = state
                    .members
                    .keys()
                    .filter(|id| !present_ids.contains(id))
                    .copied()
                    .collect();
                for id in removed {
                    if let Some(tangible) = state.members.remove(&id) {
                        actions.push(SetAction::ObjectRemoved(set.id, tangible));
                    }
                }
                // Added or moved members.
                for tangible in present {
                    match state.members.get(&tangible.id) {
                        None => {
                            state.members.insert(tangible.id, tangible.clone());
                            actions.push(SetAction::ObjectAdded(set.id, tangible.clone()));
                        }
                        Some(previous)
                            if previous.center != tangible.center
                                || previous.rotation_rad != tangible.rotation_rad =>
                        {
                            state.members.insert(tangible.id, tangible.clone());
                            actions.push(SetAction::ObjectMoved(set.id, tangible.clone()));
                        }
                        _ => {}
                    }
                }
            } else {
                // While inactive no member diffing - clear stale members so a
                // re-activation reports them as added again.
                state.members.clear();
            }
        }
        actions
    }

    /// Whether the set with the given ID is currently active.
    pub fn is_active(&self, set_id: TangibleSetId) -> bool {
        self.states.get(&set_id).is_some_and(|s| s.active)
    }

    /// All set definitions this tracker manages.
    pub fn sets(&self) -> &[TangibleSet] {
        &self.sets
    }
}

#[cfg(test)]
mod tests {
    use super::SetTracker;
    use crate::PatternId;
    use crate::RotationRad;
    use crate::SetAction;
    use crate::TangibleId;
    use crate::TangibleSet;
    use crate::TangibleSetId;
    use crate::TrackedTangible;
    use crate::Vec2;

    fn tangible(instance: u32, pattern: u32, x: f64, y: f64) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(instance),
            pattern_id: PatternId::new(pattern),
            pattern_name: format!("fig-{pattern}").into(),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(0.0),
            frames_since_match: 0,
        }
    }

    fn set(id: u32, mandatory: &[u32], additional: &[u32]) -> TangibleSet {
        TangibleSet {
            id: TangibleSetId::new(id),
            name: format!("set-{id}"),
            mandatory: mandatory.iter().map(|p| PatternId::new(*p)).collect(),
            additional: additional.iter().map(|p| PatternId::new(*p)).collect(),
        }
    }

    fn added_ids(actions: &[SetAction]) -> Vec<TangibleId> {
        actions
            .iter()
            .filter_map(|a| match a {
                SetAction::ObjectAdded(_, t) => Some(t.id),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn set_activates_only_when_all_mandatory_present() {
        let mut tracker = SetTracker::new(vec![set(1, &[1, 2], &[])]);

        // One of two mandatory members is not enough.
        assert!(tracker.update(&[tangible(1, 1, 0.0, 0.0)]).is_empty());
        assert!(!tracker.is_active(TangibleSetId::new(1)));

        // All mandatory members present: activation + one add per member.
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0), tangible(2, 2, 10.0, 0.0)]);
        assert!(tracker.is_active(TangibleSetId::new(1)));
        assert_eq!(actions[0], SetAction::SetAdded(TangibleSetId::new(1)));
        assert_eq!(added_ids(&actions).len(), 2);
    }

    #[test]
    fn hysteresis_keeps_set_active_with_partial_mandatory() {
        let mut tracker = SetTracker::new(vec![set(1, &[1, 2], &[])]);
        tracker.update(&[tangible(1, 1, 0.0, 0.0), tangible(2, 2, 10.0, 0.0)]);

        // Only one mandatory member left: set stays active, member reported
        // as removed.
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0)]);
        assert!(tracker.is_active(TangibleSetId::new(1)));
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, SetAction::SetRemoved(_)))
        );
        assert!(actions.iter().any(|a| matches!(
            a,
            SetAction::ObjectRemoved(id, _) if *id == TangibleSetId::new(1)
        )));
    }

    #[test]
    fn set_deactivates_when_last_mandatory_member_removed() {
        let mut tracker = SetTracker::new(vec![set(1, &[1], &[2])]);
        tracker.update(&[tangible(1, 1, 0.0, 0.0), tangible(2, 2, 10.0, 0.0)]);

        let actions = tracker.update(&[]);
        assert!(!tracker.is_active(TangibleSetId::new(1)));
        assert!(
            actions
                .iter()
                .any(|a| *a == SetAction::SetRemoved(TangibleSetId::new(1)))
        );
        // All known members are reported removed on deactivation.
        assert_eq!(
            actions
                .iter()
                .filter(|a| matches!(a, SetAction::ObjectRemoved(_, _)))
                .count(),
            2
        );
    }

    #[test]
    fn additional_member_emits_add_move_remove() {
        let mut tracker = SetTracker::new(vec![set(1, &[1], &[2])]);
        tracker.update(&[tangible(1, 1, 0.0, 0.0)]);

        // Additional member appears: ObjectAdded.
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0), tangible(2, 2, 10.0, 0.0)]);
        assert!(actions.iter().any(|a| matches!(
            a,
            SetAction::ObjectAdded(_, t) if t.id == TangibleId::new(2)
        )));

        // Pose change: ObjectMoved.
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0), tangible(2, 2, 20.0, 0.0)]);
        assert!(actions.iter().any(|a| matches!(
            a,
            SetAction::ObjectMoved(_, t) if t.id == TangibleId::new(2)
        )));

        // Disappears: ObjectRemoved - but the set stays active because the
        // mandatory member remains.
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0)]);
        assert!(actions.iter().any(|a| matches!(
            a,
            SetAction::ObjectRemoved(_, t) if t.id == TangibleId::new(2)
        )));
        assert!(tracker.is_active(TangibleSetId::new(1)));
    }

    #[test]
    fn additional_only_members_never_activate() {
        let mut tracker = SetTracker::new(vec![set(1, &[1], &[2])]);
        let actions = tracker.update(&[tangible(2, 2, 10.0, 0.0)]);
        assert!(actions.is_empty());
        assert!(!tracker.is_active(TangibleSetId::new(1)));
    }

    #[test]
    fn set_without_mandatory_members_never_activates() {
        let mut tracker = SetTracker::new(vec![set(1, &[], &[1])]);
        let actions = tracker.update(&[tangible(1, 1, 0.0, 0.0)]);
        assert!(actions.is_empty());
        assert!(!tracker.is_active(TangibleSetId::new(1)));
    }

    #[test]
    fn reactivation_reports_members_as_added_again() {
        let mut tracker = SetTracker::new(vec![set(1, &[1], &[])]);
        tracker.update(&[tangible(1, 1, 0.0, 0.0)]);
        tracker.update(&[]);

        // New instance (new TangibleId) re-activates the set.
        let actions = tracker.update(&[tangible(5, 1, 0.0, 0.0)]);
        assert!(
            actions
                .iter()
                .any(|a| *a == SetAction::SetAdded(TangibleSetId::new(1)))
        );
        assert_eq!(added_ids(&actions), vec![TangibleId::new(5)]);
    }
}
