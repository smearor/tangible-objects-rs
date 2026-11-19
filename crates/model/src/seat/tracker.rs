use std::collections::HashSet;

use super::SeatEvent;
use super::SeatId;
use super::SeatMarkerState;
use super::SeatName;
use super::SeatSession;
use super::SeatSessionName;
use super::TableTopSeat;
use crate::Owner;
use crate::PatternId;
use crate::RegistrationError;
use crate::TrackedSet;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Rect;
use smearor_commons_layout::SeatPose;

/// Session registry and per-frame seat tracker
/// (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
///
/// Holds the claimed [`SeatSession`]s with their rosters and, fed with
/// each frame's [`TrackedSet`], binds marker tangibles to seats,
/// re-derives [`SeatPose`]s and reports the roster/anchor changes as
/// [`SeatEvent`]s - the seat-domain counterpart of
/// [`crate::SetTracker`].
///
/// Ownership follows the profile-claim contract: a session has exactly
/// one owner, only the owner may mutate its roster, and
/// [`Self::remove_by_owner`] implements the `UnregisterOwner`/disconnect
/// cleanup. The absent hysteresis is sticky by design - an `Absent`
/// seat keeps its last pose until the marker returns or the
/// application removes the seat.
#[derive(Debug)]
pub struct SeatTracker {
    /// Claimed sessions with their seat rosters.
    sessions: Vec<SeatSession>,
    /// Physical surface bounds for the marker-position derivation;
    /// `None` until the device's calibrated millimeter bounds are
    /// known - binding still works, poses stay as provided.
    bounds: Option<Rect<Millimeters>>,
    /// Next seat ID to assign - never reused.
    next_seat_id: SeatId,
}

impl Default for SeatTracker {
    /// Empty tracker, seat IDs starting at 1.
    fn default() -> Self {
        Self::new()
    }
}

impl SeatTracker {
    /// Creates an empty tracker - no sessions, no bounds yet.
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            bounds: None,
            next_seat_id: SeatId::new(1),
        }
    }

    /// Sets the physical surface bounds the marker positions are
    /// projected against (`SeatPose::from_position`). Called once the
    /// device's calibrated millimeter bounds are known.
    pub fn set_bounds(&mut self, bounds: Rect<Millimeters>) {
        self.bounds = Some(bounds);
    }

    /// All claimed sessions with their rosters.
    pub fn sessions(&self) -> &[SeatSession] {
        &self.sessions
    }

    /// The session registered under `name`, if any.
    pub fn session(&self, name: &SeatSessionName) -> Option<&SeatSession> {
        self.sessions.iter().find(|s| &s.name == name)
    }

    /// Claims or creates a session for `owner` - the seat-domain
    /// counterpart of `Engine::claim_pattern`: idempotent for the same
    /// owner, [`RegistrationError::SessionAlreadyOwned`] for a foreign
    /// one. Returns the session and whether it was newly created.
    pub fn claim(
        &mut self,
        name: SeatSessionName,
        owner: Owner,
    ) -> Result<(SeatSession, bool), RegistrationError> {
        if let Some(existing) = self.sessions.iter().find(|s| s.name == name) {
            return if existing.owner == owner {
                Ok((existing.clone(), false))
            } else {
                Err(RegistrationError::SessionAlreadyOwned(
                    name,
                    existing.owner.clone(),
                ))
            };
        }
        let session = SeatSession {
            name,
            owner,
            seats: Vec::new(),
        };
        self.sessions.push(session.clone());
        Ok((session, true))
    }

    /// Releases a claimed session - only the owner may release
    /// ([`RegistrationError::SessionNotOwnedBy`]). Returns the removed
    /// session; its seats are gone with it.
    pub fn release(
        &mut self,
        name: &SeatSessionName,
        owner: &Owner,
    ) -> Result<SeatSession, RegistrationError> {
        let Some(index) = self.sessions.iter().position(|s| &s.name == name) else {
            return Err(RegistrationError::SessionNotFound(name.clone()));
        };
        if self.sessions[index].owner != *owner {
            return Err(RegistrationError::SessionNotOwnedBy(
                name.clone(),
                owner.clone(),
            ));
        }
        Ok(self.sessions.remove(index))
    }

    /// Adds a seat to a claimed session - marker-backed
    /// (`marker: Some`, starts `Absent` until the first bind) or
    /// virtual (`marker: None`, keeps `pose`). Only the session owner
    /// may add seats; names are unique within the session.
    pub fn add_seat(
        &mut self,
        session: &SeatSessionName,
        owner: &Owner,
        name: SeatName,
        marker: Option<PatternId>,
        pose: Option<SeatPose>,
    ) -> Result<TableTopSeat, RegistrationError> {
        let Some(index) = self.sessions.iter().position(|s| &s.name == session) else {
            return Err(RegistrationError::SessionNotFound(session.clone()));
        };
        let target = &mut self.sessions[index];
        if target.owner != *owner {
            return Err(RegistrationError::SessionNotOwnedBy(
                session.clone(),
                owner.clone(),
            ));
        }
        if target.seats.iter().any(|seat| seat.name == name) {
            return Err(RegistrationError::DuplicateSeat(name));
        }
        let seat = TableTopSeat {
            id: self.next_seat_id,
            name,
            session: session.clone(),
            owner: target.owner.clone(),
            marker,
            tangible: None,
            pose: pose.unwrap_or_default(),
            state: if marker.is_some() {
                SeatMarkerState::Absent
            } else {
                SeatMarkerState::Virtual
            },
        };
        self.next_seat_id = self.next_seat_id.next();
        target.seats.push(seat.clone());
        Ok(seat)
    }

    /// Mutates a seat addressed by its globally unique ID - `None`
    /// fields keep their value; `marker`'s inner `None` clears the
    /// marker so the seat turns virtual. Only the session owner may
    /// mutate. Returns the updated seat and whether its pose changed.
    pub fn update_seat(
        &mut self,
        id: SeatId,
        owner: &Owner,
        name: Option<SeatName>,
        marker: Option<Option<PatternId>>,
        pose: Option<SeatPose>,
    ) -> Result<(TableTopSeat, bool), RegistrationError> {
        let Some((session_index, seat_index)) = self.find(id) else {
            return Err(RegistrationError::SeatNotFound(id));
        };
        let session = &mut self.sessions[session_index];
        if session.owner != *owner {
            return Err(RegistrationError::SessionNotOwnedBy(
                session.name.clone(),
                owner.clone(),
            ));
        }
        if let Some(name) = &name
            && session
                .seats
                .iter()
                .any(|seat| seat.id != id && seat.name == *name)
        {
            return Err(RegistrationError::DuplicateSeat(name.clone()));
        }
        let seat = &mut session.seats[seat_index];
        if let Some(name) = name {
            seat.name = name;
        }
        if let Some(marker) = marker
            && seat.marker != marker
        {
            seat.marker = marker;
            seat.tangible = None;
            seat.state = if marker.is_some() {
                SeatMarkerState::Absent
            } else {
                SeatMarkerState::Virtual
            };
        }
        let mut pose_changed = false;
        if let Some(pose) = pose
            && seat.pose != pose
        {
            seat.pose = pose;
            pose_changed = true;
        }
        Ok((seat.clone(), pose_changed))
    }

    /// Removes a seat addressed by its globally unique ID - only the
    /// session owner may remove. Returns the removed seat.
    pub fn remove_seat(
        &mut self,
        id: SeatId,
        owner: &Owner,
    ) -> Result<TableTopSeat, RegistrationError> {
        let Some((session_index, seat_index)) = self.find(id) else {
            return Err(RegistrationError::SeatNotFound(id));
        };
        let session = &mut self.sessions[session_index];
        if session.owner != *owner {
            return Err(RegistrationError::SessionNotOwnedBy(
                session.name.clone(),
                owner.clone(),
            ));
        }
        Ok(session.seats.remove(seat_index))
    }

    /// Drops every session owned by `owner` - the `UnregisterOwner`
    /// and D-Bus-disconnect cleanup. Returns the removed sessions so
    /// the caller can report their release.
    pub fn remove_by_owner(&mut self, owner: &Owner) -> Vec<SeatSession> {
        let (removed, kept): (Vec<_>, Vec<_>) = self
            .sessions
            .drain(..)
            .partition(|session| &session.owner == owner);
        self.sessions = kept;
        removed
    }

    /// Feeds the current frame's tracked tangibles and returns the
    /// seat events since the previous update - binds marker instances,
    /// re-derives anchored poses and flips the anchor state.
    ///
    /// Per session a tangible anchors at most one seat: the incumbent
    /// binding wins, otherwise seats pick the smallest free matching
    /// [`TangibleId`] in roster order (deterministic spawn order).
    pub fn update(&mut self, tracked: TrackedSet<'_>) -> Vec<SeatEvent> {
        let mut events = Vec::new();
        for session in &mut self.sessions {
            let mut bound = HashSet::new();
            for seat in &mut session.seats {
                let Some(marker) = seat.marker else {
                    continue;
                };
                let candidate = seat
                    .tangible
                    .and_then(|id| {
                        tracked
                            .iter()
                            .find(|t| t.id == id && t.pattern_id == marker)
                    })
                    .or_else(|| {
                        tracked
                            .iter()
                            .filter(|t| t.pattern_id == marker && !bound.contains(&t.id))
                            .min_by_key(|t| t.id)
                    });
                match candidate {
                    Some(tangible) => {
                        bound.insert(tangible.id);
                        let rebound = seat.state != SeatMarkerState::Anchored
                            || seat.tangible != Some(tangible.id);
                        seat.tangible = Some(tangible.id);
                        let mut moved = false;
                        if let Some(bounds) = self.bounds {
                            let pose = SeatPose::from_position(
                                tangible.center,
                                bounds,
                                tangible.rotation_rad,
                            );
                            if seat.pose != pose {
                                seat.pose = pose;
                                moved = true;
                            }
                        }
                        if rebound {
                            seat.state = SeatMarkerState::Anchored;
                            events.push(SeatEvent::MarkerBound(seat.clone()));
                        } else if moved {
                            events.push(SeatEvent::Moved(seat.clone()));
                        }
                    }
                    None => {
                        seat.tangible = None;
                        if seat.state == SeatMarkerState::Anchored {
                            seat.state = SeatMarkerState::Absent;
                            events.push(SeatEvent::MarkerAbsent(seat.clone()));
                        } else {
                            seat.state = SeatMarkerState::Absent;
                        }
                    }
                }
            }
        }
        events
    }

    /// Locates a seat by ID - `(session index, seat index)`.
    fn find(&self, id: SeatId) -> Option<(usize, usize)> {
        self.sessions.iter().enumerate().find_map(|(s, session)| {
            session
                .seats
                .iter()
                .position(|seat| seat.id == id)
                .map(|i| (s, i))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SeatTracker;
    use crate::DEFAULT_SPAWN_STABILITY;
    use crate::FrameCount;
    use crate::Owner;
    use crate::PatternId;
    use crate::RegistrationError;
    use crate::SeatEvent;
    use crate::SeatId;
    use crate::SeatMarkerState;
    use crate::SeatName;
    use crate::SeatSessionName;
    use crate::TangibleId;
    use crate::TrackedTangible;
    use smearor_commons_geometry::Millimeters;
    use smearor_commons_geometry::Rect;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;
    use smearor_commons_layout::SeatPose;
    use smearor_commons_layout::TableSide;

    const OWNER: &str = "app";
    const OTHER: &str = "other";

    fn bounds() -> Rect<Millimeters> {
        Rect::new(Vec2::new(0.0, 0.0), Vec2::new(400.0, 300.0))
    }

    fn owner(name: &str) -> Owner {
        Owner::new(name)
    }

    fn session_name(name: &str) -> SeatSessionName {
        SeatSessionName::new(name)
    }

    fn tangible(instance: u32, pattern: u32, x: f64, y: f64, rotation: f64) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(instance),
            pattern_id: PatternId::new(pattern),
            pattern_name: format!("fig-{pattern}").into(),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(rotation),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            frames_since_match: FrameCount::new(0),
        }
    }

    fn tracker_with_session() -> SeatTracker {
        let mut tracker = SeatTracker::new();
        tracker.set_bounds(bounds());
        tracker
            .claim(session_name("game"), owner(OWNER))
            .expect("claim");
        tracker
    }

    fn add_marker_seat(tracker: &mut SeatTracker, name: &str, pattern: u32) -> SeatId {
        tracker
            .add_seat(
                &session_name("game"),
                &owner(OWNER),
                SeatName::new(name),
                Some(PatternId::new(pattern)),
                None,
            )
            .expect("add_seat")
            .id
    }

    #[test]
    fn claim_creates_session_and_is_idempotent_for_owner() {
        let mut tracker = SeatTracker::new();
        let (session, created) = tracker
            .claim(session_name("game"), owner(OWNER))
            .expect("claim");
        assert!(created);
        assert_eq!(session.seats.len(), 0);

        let (session, created) = tracker
            .claim(session_name("game"), owner(OWNER))
            .expect("re-claim");
        assert!(!created);
        assert_eq!(session.name, session_name("game"));
        assert_eq!(tracker.sessions().len(), 1);
    }

    #[test]
    fn claim_rejects_foreign_owner() {
        let mut tracker = SeatTracker::new();
        tracker
            .claim(session_name("game"), owner(OWNER))
            .expect("claim");
        let result = tracker.claim(session_name("game"), owner(OTHER));
        assert!(matches!(
            result,
            Err(RegistrationError::SessionAlreadyOwned(name, existing))
                if name == session_name("game") && existing == owner(OWNER)
        ));
    }

    #[test]
    fn release_is_owner_only_and_removes_seats() {
        let mut tracker = tracker_with_session();
        let seat = add_marker_seat(&mut tracker, "seat-1", 1);
        assert!(matches!(
            tracker.release(&session_name("missing"), &owner(OWNER)),
            Err(RegistrationError::SessionNotFound(_))
        ));
        assert!(matches!(
            tracker.release(&session_name("game"), &owner(OTHER)),
            Err(RegistrationError::SessionNotOwnedBy(_, _))
        ));
        let session = tracker
            .release(&session_name("game"), &owner(OWNER))
            .expect("release");
        assert_eq!(session.seats[0].id, seat);
        assert!(tracker.sessions().is_empty());
    }

    #[test]
    fn add_seat_validates_session_owner_and_name() {
        let mut tracker = tracker_with_session();
        assert!(matches!(
            tracker.add_seat(
                &session_name("missing"),
                &owner(OWNER),
                SeatName::new("seat"),
                None,
                None
            ),
            Err(RegistrationError::SessionNotFound(_))
        ));
        assert!(matches!(
            tracker.add_seat(
                &session_name("game"),
                &owner(OTHER),
                SeatName::new("other-seat"),
                None,
                None
            ),
            Err(RegistrationError::SessionNotOwnedBy(_, _))
        ));
        tracker
            .add_seat(
                &session_name("game"),
                &owner(OWNER),
                SeatName::new("seat"),
                None,
                None,
            )
            .expect("add");
        assert!(matches!(
            tracker.add_seat(
                &session_name("game"),
                &owner(OWNER),
                SeatName::new("seat"),
                None,
                None
            ),
            Err(RegistrationError::DuplicateSeat(_))
        ));
    }

    #[test]
    fn virtual_seat_keeps_pose_and_skips_binding() {
        let mut tracker = tracker_with_session();
        let pose =
            SeatPose::from_position(Vec2::new(100.0, 150.0), bounds(), RotationRad::new(0.7));
        let seat = tracker
            .add_seat(
                &session_name("game"),
                &owner(OWNER),
                SeatName::new("virtual"),
                None,
                Some(pose),
            )
            .expect("add");
        assert_eq!(seat.state, SeatMarkerState::Virtual);
        assert_eq!(seat.pose, pose);

        // Frames never touch a virtual seat, not even with a matching
        // tangible around.
        let events = tracker.update((&[tangible(1, 1, 5.0, 5.0, 0.0)]).into());
        assert!(events.is_empty());
        let seat = &tracker
            .session(&session_name("game"))
            .expect("session")
            .seats[0];
        assert_eq!(seat.state, SeatMarkerState::Virtual);
        assert_eq!(seat.pose, pose);
        assert_eq!(seat.tangible, None);
    }

    #[test]
    fn marker_binds_and_derives_pose() {
        let mut tracker = tracker_with_session();
        add_marker_seat(&mut tracker, "seat-1", 1);

        // Center of the bottom edge, facing diagonally inward.
        let events = tracker.update((&[tangible(7, 1, 200.0, 290.0, 0.5)]).into());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SeatEvent::MarkerBound(_)));
        let seat = &tracker
            .session(&session_name("game"))
            .expect("session")
            .seats[0];
        assert_eq!(seat.state, SeatMarkerState::Anchored);
        assert_eq!(seat.tangible, Some(TangibleId::new(7)));
        assert_eq!(seat.pose.seat.side, TableSide::Bottom);
        assert_eq!(seat.pose.orientation, RotationRad::new(0.5));
    }

    #[test]
    fn moved_marker_derives_seat_moved() {
        let mut tracker = tracker_with_session();
        add_marker_seat(&mut tracker, "seat-1", 1);
        tracker.update((&[tangible(7, 1, 200.0, 290.0, 0.0)]).into());

        let events = tracker.update((&[tangible(7, 1, 210.0, 290.0, 0.0)]).into());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SeatEvent::Moved(_)));
    }

    #[test]
    fn absent_marker_keeps_pose_and_rebinds() {
        let mut tracker = tracker_with_session();
        add_marker_seat(&mut tracker, "seat-1", 1);
        tracker.update((&[tangible(7, 1, 200.0, 290.0, 0.5)]).into());
        let anchored_pose = tracker.session(&session_name("game")).expect("s").seats[0].pose;

        // Marker vanishes: Absent, pose retained, no premature removal.
        let events = tracker.update((&[]).into());
        assert!(matches!(events[0], SeatEvent::MarkerAbsent(_)));
        let seat = &tracker.session(&session_name("game")).expect("s").seats[0];
        assert_eq!(seat.state, SeatMarkerState::Absent);
        assert_eq!(seat.pose, anchored_pose);
        assert_eq!(seat.tangible, None);

        // Second absent frame: hysteresis is sticky, no new event.
        assert!(tracker.update((&[]).into()).is_empty());
        assert_eq!(
            tracker.session(&session_name("game")).expect("s").seats[0].state,
            SeatMarkerState::Absent
        );

        // Marker returns (new instance): rebinds, derives fresh pose.
        let events = tracker.update((&[tangible(9, 1, 250.0, 290.0, 0.0)]).into());
        assert!(matches!(events[0], SeatEvent::MarkerBound(_)));
        let seat = &tracker.session(&session_name("game")).expect("s").seats[0];
        assert_eq!(seat.state, SeatMarkerState::Anchored);
        assert_eq!(seat.tangible, Some(TangibleId::new(9)));
        assert_eq!(seat.pose.orientation, RotationRad::new(0.0));
    }

    #[test]
    fn same_marker_pattern_binds_distinct_instances() {
        let mut tracker = tracker_with_session();
        add_marker_seat(&mut tracker, "seat-1", 1);
        add_marker_seat(&mut tracker, "seat-2", 1);

        tracker.update(
            (&[
                tangible(3, 1, 10.0, 10.0, 0.0),
                tangible(5, 1, 390.0, 10.0, 0.0),
            ])
                .into(),
        );
        let session = tracker.session(&session_name("game")).expect("session");
        // Roster order wins the smallest free instance.
        assert_eq!(session.seats[0].tangible, Some(TangibleId::new(3)));
        assert_eq!(session.seats[1].tangible, Some(TangibleId::new(5)));
    }

    #[test]
    fn update_seat_changes_fields_and_reports_pose_change() {
        let mut tracker = tracker_with_session();
        let id = add_marker_seat(&mut tracker, "seat-1", 1);
        assert!(matches!(
            tracker.update_seat(id, &owner(OTHER), None, None, None),
            Err(RegistrationError::SessionNotOwnedBy(_, _))
        ));

        let pose = SeatPose::from_position(Vec2::new(50.0, 50.0), bounds(), RotationRad::new(0.3));
        let (seat, moved) = tracker
            .update_seat(
                id,
                &owner(OWNER),
                Some(SeatName::new("renamed")),
                None,
                Some(pose),
            )
            .expect("update");
        assert_eq!(seat.name, SeatName::new("renamed"));
        assert!(moved);

        // Clearing the marker turns the seat virtual.
        let (seat, _) = tracker
            .update_seat(id, &owner(OWNER), None, Some(None), None)
            .expect("update");
        assert_eq!(seat.marker, None);
        assert_eq!(seat.state, SeatMarkerState::Virtual);
    }

    #[test]
    fn remove_seat_is_owner_only() {
        let mut tracker = tracker_with_session();
        let id = add_marker_seat(&mut tracker, "seat-1", 1);
        assert!(matches!(
            tracker.remove_seat(id, &owner(OTHER)),
            Err(RegistrationError::SessionNotOwnedBy(_, _))
        ));
        let seat = tracker.remove_seat(id, &owner(OWNER)).expect("remove");
        assert_eq!(seat.id, id);
        assert!(
            tracker
                .session(&session_name("game"))
                .expect("s")
                .seats
                .is_empty()
        );
        assert!(matches!(
            tracker.remove_seat(id, &owner(OWNER)),
            Err(RegistrationError::SeatNotFound(_))
        ));
    }

    #[test]
    fn remove_by_owner_drops_only_their_sessions() {
        let mut tracker = SeatTracker::new();
        tracker
            .claim(session_name("game"), owner(OWNER))
            .expect("claim");
        tracker
            .claim(session_name("board"), owner(OTHER))
            .expect("claim");
        tracker
            .add_seat(
                &session_name("game"),
                &owner(OWNER),
                SeatName::new("s"),
                None,
                None,
            )
            .expect("add");

        let removed = tracker.remove_by_owner(&owner(OWNER));
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].seats.len(), 1);
        assert_eq!(tracker.sessions().len(), 1);
        assert_eq!(tracker.sessions()[0].name, session_name("board"));
    }
}
