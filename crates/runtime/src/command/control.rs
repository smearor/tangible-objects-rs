use crate::AddSeatRequest;
use crate::UpdateSeatRequest;

use smearor_commons_color::AlphaPercent;
use smearor_commons_geometry::Millimeters;
use smearor_commons_layout::SeatPose;
use std::path::PathBuf;
use tangible_objects_engine::LegacyMode;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FigureColor;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::HandName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::ProfileClaimed;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::ProfileLookup;
use tangible_objects_model::ProfileReleased;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;
use tangible_objects_model::RegistrationApplied;
use tangible_objects_model::RegistrationError;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSession;
use tangible_objects_model::SeatSessionName;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;
use tangible_objects_model::StageName;
use tangible_objects_model::TableTopSeat;
use tokio::sync::oneshot;

/// Control command sent by an interface back to the daemon's device loop.
///
/// Commands are drained by the runtime after each published frame
/// ([`crate::EngineRuntime::try_recv_command`]); the concrete behavior is
/// implemented where the owning functionality lands (e.g. filter mode in
/// Phase 4, remote training with the interface crates).
#[derive(Debug)]
pub enum ControlCommand {
    /// Switch the filter proxy's suppression policy
    /// ([`FilterMode::Off`] = transparent passthrough). `stability`
    /// overrides the consecutive-matched-frames gate when `Some`; `None`
    /// keeps the current value.
    SetFilterMode {
        /// New suppression policy.
        mode: FilterMode,
        /// Consecutive frames a slot must match before suppression starts.
        stability: Option<FrameCount>,
        /// Scope of the stage to reconfigure (EVENT_PIPELINE.md phase 7):
        /// `Some` applies the policy to that stage only, `None`
        /// broadcasts to all tangible stages. Positional stages always
        /// ignore `SetFilterMode`.
        stage: Option<StageName>,
    },
    /// Create or claim a pipeline stage for `name` before any `stage =
    /// "name"` pattern registers - the stage persists until
    /// `UnregisterStage`, independent of pattern lifetimes
    /// (EVENT_PIPELINE.md phase 7). Lets a client pre-configure a
    /// stage's `SetFilterMode` policy.
    RegisterStage {
        /// Stage scope name.
        name: StageName,
    },
    /// Remove the pipeline stage for `name` - its suppression state
    /// dies with it; patterns of that scope keep matching but are no
    /// longer stage-suppressed until a new stage exists.
    UnregisterStage {
        /// Stage scope name.
        name: StageName,
    },
    /// Capture a new figure profile with the given name.
    TrainProfile(PatternName),
    /// Capture a new spell gesture: the next
    /// [`tangible_objects_engine`] training repetitions become the
    /// template; the finished spell registers persistently
    /// (SPELLS.md's learn-by-repetition).
    TrainSpell {
        /// Spell name - addresses it for unregister and the store.
        name: SpellName,
        /// Arming the finished spell carries.
        arming: SpellArming,
    },
    /// Start a recording session writing `<base_path>.jsonl` (frames) and
    /// `<base_path>.evtest` (raw events).
    StartRecording {
        /// Base path of the recording artifacts, without extension.
        base_path: PathBuf,
        /// Ground-truth label applied to recorded frames until changed.
        label: Option<RecordLabel>,
    },
    /// Stop the active recording session.
    StopRecording,
    /// Change the ground-truth label applied to subsequent recorded frames.
    SetRecordLabel(Option<RecordLabel>),
    /// Replace the missing-pin completion configuration at runtime
    /// (TRACKING_STABILIZATION.md). Applies to matching and suppression in
    /// all modes from the next frame on.
    SetCompletion(CompletionConfig),
    /// Edge-margin ghost filter width - touches closer than `margin` to
    /// the surface edge are dropped before matching (GUI.md §5).
    /// Applies in every mode; `0` disables the filter.
    SetEdgeMargin(Millimeters),
    /// Legacy single-touch event policy on the filter proxy's virtual
    /// device (`Auto`/`Always`/`Never`, GUI.md §5). Only the
    /// `--filter` pipeline emits legacy events; other modes publish it
    /// as display state.
    SetLegacyMode(LegacyMode),
    /// Reset the accumulated [`crate::SessionMetrics`] counters - e.g.
    /// between automated test runs.
    ResetMetrics,
    /// Display a user-facing message in the GUI ("Legen Sie Figur 10
    /// auf") - interfaces steer the operator during test series.
    /// `None`-equivalent empty string clears it.
    ShowMessage(String),
    /// Register a profile or field set at runtime (DYNAMIC_PROFILES.md).
    /// Registrations whose fingerprint collides with an already-loaded
    /// profile are rejected in the device loop.
    Register {
        /// What to register - boxed: `ProfileFile` makes this variant
        /// large; boxing keeps `ControlCommand` small for the channel.
        registration: Box<Registration>,
        /// Optional application name the registration belongs to -
        /// `UnregisterOwner` removes every registration of an owner;
        /// D-Bus binds registrations to the caller's unique name for
        /// automatic disconnect cleanup.
        owner: Option<Owner>,
        /// `Session` (default) or `Persistent` - persistent registrations
        /// are written to the daemon's runtime store on apply.
        lifetime: Lifetime,
        /// Receives the applied outcome - `Ok` carries what was
        /// registered (the profile's assigned `PatternId`), the typed
        /// rejection on failure - so interface callers answer with the
        /// real result instead of a fire-and-forget "queued".
        reply: oneshot::Sender<Result<RegistrationApplied, RegistrationError>>,
    },
    /// Compares a candidate profile's fingerprint against the
    /// registered patterns - the `profileLookup`/`find_profile`
    /// check applications run before `Register` to tell "already
    /// have it" from "free to register". Runs the same collision
    /// check `Register` rejects on and reports whether the match is
    /// elementwise-identical.
    LookupProfile {
        /// Candidate profile - boxed like `Register`.
        profile: Box<ProfileFile>,
        /// Receives the comparison outcome.
        reply: oneshot::Sender<ProfileLookup>,
    },
    /// Claims an owner-less profile for `owner` (`claimProfile`) - an
    /// application adopts a statically loaded or released profile so
    /// `UnregisterOwner`/disconnect cleanup releases it back to owner-less instead of unregistering it. Re-claiming an
    /// already-owned profile succeeds only for the same owner.
    /// Profiles are addressed by ID - `LookupProfile`'s match carries
    /// it - since profile names are not unique.
    ClaimProfile {
        /// ID of the profile to claim.
        id: PatternId,
        /// Owning application taking over the profile.
        owner: Owner,
        /// Activate the profile in the same operation - for
        /// "inactive by default" profiles the owner needs only
        /// while it runs. Cleanup reverts the activation.
        enable: bool,
        /// Receives the claimed profile or the typed rejection.
        reply: oneshot::Sender<Result<ProfileClaimed, RegistrationError>>,
    },
    /// Releases a claimed profile back to owner-less state
    /// (`releaseProfile`) - the profile stays registered and keeps
    /// matching, it just leaves the owner's cleanup scope.
    ReleaseProfile {
        /// ID of the profile to release.
        id: PatternId,
        /// Owner giving up its claim - only the current owner may
        /// release.
        owner: Owner,
        /// Deactivate the profile in the same operation - the
        /// counterpart of `ClaimProfile`'s `enable`.
        disable: bool,
        /// Receives the released profile or the typed rejection.
        reply: oneshot::Sender<Result<ProfileReleased, RegistrationError>>,
    },
    /// Replace a registered pattern's geometry/name while keeping its
    /// ID (`UpdateProfile`). Applies to new matches; tracked tangibles
    /// keep their recorded pose.
    UpdateProfile {
        /// Pattern to replace.
        id: PatternId,
        /// New profile definition - boxed like `Register`.
        profile: Box<ProfileFile>,
    },
    /// Remove a dynamically registered pattern by its pattern ID. The ID
    /// is never reused.
    UnregisterPattern {
        /// Pattern to remove.
        id: PatternId,
    },
    /// Remove a field set by name.
    UnregisterFieldSet {
        /// Field set to remove.
        name: FieldSetName,
    },
    /// Remove a dead zone by name.
    UnregisterDeadZone {
        /// Dead zone to remove.
        name: DeadZoneName,
        /// Also delete the zone's `dead_zones.d` store entry. Interface
        /// callers pass `true` (remove means gone for good); the GUI
        /// card passes `false` so removals stay session-scoped until
        /// its Save button commits them via [`Self::PersistDeadZones`].
        unpersist: bool,
    },
    /// Reconcile `dead_zones.d` with the live dead-zone set: writes a
    /// store file per registered zone and removes store entries whose
    /// zone is no longer registered - the dead-zones card's Save
    /// button, committing session adds and removals in one shot.
    PersistDeadZones,
    /// Remove a registered spell by name (`Registration::Spell`
    /// undo, SPELLS.md).
    UnregisterSpell {
        /// Spell to remove.
        name: SpellName,
    },
    /// Remove a registered handprint by name
    /// (`Registration::HandProfile` undo, SPELLS.md's per-user
    /// spells).
    UnregisterHandProfile {
        /// Handprint to remove.
        name: HandName,
    },
    /// Remove every pattern, field set and dead zone owned by `owner` -
    /// used for explicit app cleanup and D-Bus disconnect handling.
    UnregisterOwner {
        /// Owner name (or the caller's unique bus name on D-Bus).
        owner: Owner,
    },
    /// Show or hide the daemon-embedded GUI window (`crates/gtk`,
    /// `--gui` mode). Part of the command vocabulary on all builds -
    /// daemons without the `gui` feature ignore it (the runtime's
    /// `gui_visible` state is feature-gated).
    SetGuiVisible(bool),
    /// Quarter-turn the GUI canvas renders (`--display-rotation`).
    /// Same availability as [`Self::SetGuiVisible`] - headless daemons
    /// ignore it.
    SetDisplayRotation(DisplayRotation),
    /// Show or hide the dead-zone overlay on the GUI canvas. Same
    /// availability as [`Self::SetGuiVisible`] - headless daemons
    /// ignore it.
    SetDeadZonesVisible(bool),
    /// Show or hide the live spell-gesture overlay
    /// (SPELLS.md debug visualization).
    SetSpellsVisible(bool),
    /// Canvas background opacity (`--background-alpha`). Same
    /// availability as [`Self::SetGuiVisible`] - headless daemons
    /// ignore it.
    SetBackgroundAlpha(AlphaPercent),
    /// Show or hide the edge-margin overlay on the GUI canvas -
    /// the area where edge-ghost contacts are ingest-filtered.
    /// Same availability as [`Self::SetGuiVisible`].
    SetEdgeMarginVisible(bool),
    /// Show or hide the field-set overlay on the GUI canvas - the
    /// occupancy regions of every registered field set. Same
    /// availability as [`Self::SetGuiVisible`].
    SetFieldsVisible(bool),
    /// Show or hide the ruler overlay on the GUI canvas - pixel ticks
    /// on one edge, millimeter ticks on the other. Same availability
    /// as [`Self::SetGuiVisible`].
    SetRulerVisible(bool),
    /// Show or hide the grid overlay on the GUI canvas - a millimeter
    /// lattice across the whole surface. Same availability as
    /// [`Self::SetGuiVisible`].
    SetGridVisible(bool),
    /// Show or hide the positions overlay on the GUI canvas - mm and
    /// pixel coordinates of tangibles, touch points and field-set
    /// fields. Same availability as [`Self::SetGuiVisible`].
    SetPositionsVisible(bool),
    /// Let touch events pass through the GTK window to the desktop
    /// beneath - except where a GUI element (control bar) is hit.
    /// Same availability as [`Self::SetGuiVisible`]; Wayland only.
    SetTouchPassthrough(bool),
    /// Enable or disable a registered pattern at runtime - disabled
    /// profiles stay registered but produce no matches and spawn no
    /// tangibles. The GUI toggles it from the profiles card.
    SetProfileEnabled {
        /// Pattern to toggle.
        id: PatternId,
        /// New enabled state.
        enabled: bool,
    },
    /// Override a registered pattern's declared render color at
    /// runtime - session-only (the profile file on disk keeps its
    /// `color`). The profiles card's color picker sends it.
    SetProfileColor {
        /// Pattern to recolor.
        id: PatternId,
        /// New render color.
        color: FigureColor,
    },
    /// Enable or disable a registered field set at runtime - disabled
    /// sets keep their registration but track no occupancy and
    /// suppress nothing. The GUI toggles it from the field-sets card.
    SetFieldSetEnabled {
        /// Field set to toggle.
        name: FieldSetName,
        /// New enabled state.
        enabled: bool,
    },
    /// Persists every registered pattern's runtime `enabled`/`color`
    /// overrides back into its profile file on disk - the profiles
    /// card's Save button. The `profiles.d` runtime store is always
    /// included; session-only patterns without a file are skipped.
    SaveProfiles {
        /// Profile source directory to scan (`--profiles`); `None`
        /// falls back to the default `profiles/` directory.
        profiles_dir: Option<PathBuf>,
        /// Renames the profiles card's name entries collected - applied
        /// to the engine before the file rewrite so each file lands at
        /// `<new-name>.toml`. Patterns keep their `PatternId`.
        renames: Vec<(PatternId, PatternName)>,
    },
    /// Claims or creates a seat session for `owner`
    /// (`claimSeatSession`, TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md) - the
    /// seat-domain counterpart of [`Self::ClaimProfile`]: idempotent
    /// for the same owner, rejected for a foreign one, released with
    /// all its seats by `UnregisterOwner`/disconnect cleanup.
    ClaimSeatSession {
        /// Session to claim - created when no session of this name
        /// exists.
        name: SeatSessionName,
        /// Owning application taking over the session.
        owner: Owner,
        /// Receives the claimed session or the typed rejection.
        reply: oneshot::Sender<Result<SeatSession, RegistrationError>>,
    },
    /// Releases a claimed seat session - only the current owner may
    /// release (`SESSION_NOT_OWNED_BY`). Its seats go with it; each
    /// emits `SEAT_REMOVED` before `SESSION_RELEASED`.
    ReleaseSeatSession {
        /// Session to release.
        name: SeatSessionName,
        /// Owner giving up its claim.
        owner: Owner,
        /// Receives the released session or the typed rejection.
        reply: oneshot::Sender<Result<SeatSession, RegistrationError>>,
    },
    /// Adds a seat to a claimed session (`addSeat`) - marker-backed
    /// (`marker: Some`, pose derived once an instance binds) or
    /// virtual (`marker: None`, application-provided `pose`). The
    /// device loop validates `marker` against the registered patterns
    /// (`MARKER_PROFILE_NOT_FOUND`); `owner` must own the session.
    AddSeat(Box<AddSeatRequest>),
    /// Mutates a seat (`updateSeat`) - `None` fields keep their
    /// value; `marker`'s inner `None` clears the marker so the seat
    /// turns virtual. `pose` overrides the pose - honored for virtual
    /// seats; a bound marker re-derives it next frame.
    UpdateSeat(Box<UpdateSeatRequest>),
    /// Removes a seat from its session (`removeSeat`) - emits
    /// `SEAT_REMOVED`.
    RemoveSeat {
        /// Seat to remove - addressed by its globally unique ID.
        seat: SeatId,
        /// Owner removing the seat - must own its session.
        owner: Owner,
        /// Receives the removed seat or the typed rejection.
        reply: oneshot::Sender<Result<TableTopSeat, RegistrationError>>,
    },
    /// Show or hide the seat overlay on the GUI canvas - same
    /// availability as [`Self::SetFieldsVisible`]: headless daemons
    /// ignore it.
    SetSeatsVisible(bool),
}

impl ControlCommand {
    /// Builds a `Register` command plus the receiver its caller awaits
    /// for the applied outcome. Internal callers that never await simply
    /// drop the receiver - `reply.send` in the device loop tolerates a
    /// closed channel.
    pub fn register(
        registration: Registration,
        owner: Option<Owner>,
        lifetime: Lifetime,
    ) -> (
        Self,
        oneshot::Receiver<Result<RegistrationApplied, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (
            Self::Register {
                registration: Box::new(registration),
                owner,
                lifetime,
                reply,
            },
            rx,
        )
    }

    /// Builds a `LookupProfile` command plus the receiver its caller
    /// awaits for the comparison outcome.
    pub fn lookup_profile(profile: ProfileFile) -> (Self, oneshot::Receiver<ProfileLookup>) {
        let (reply, rx) = oneshot::channel();
        (
            Self::LookupProfile {
                profile: Box::new(profile),
                reply,
            },
            rx,
        )
    }

    /// Builds a `ClaimProfile` command plus the receiver its caller
    /// awaits for the claimed profile or the typed rejection.
    /// `enable` activates the profile in the same operation.
    pub fn claim_profile(
        id: PatternId,
        owner: Owner,
        enable: bool,
    ) -> (
        Self,
        oneshot::Receiver<Result<ProfileClaimed, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (
            Self::ClaimProfile {
                id,
                owner,
                enable,
                reply,
            },
            rx,
        )
    }

    /// Builds a `ReleaseProfile` command plus the receiver its caller
    /// awaits for the released profile or the typed rejection.
    /// `disable` deactivates the profile in the same operation.
    pub fn release_profile(
        id: PatternId,
        owner: Owner,
        disable: bool,
    ) -> (
        Self,
        oneshot::Receiver<Result<ProfileReleased, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (
            Self::ReleaseProfile {
                id,
                owner,
                disable,
                reply,
            },
            rx,
        )
    }

    /// Builds a `ClaimSeatSession` command plus the receiver its caller
    /// awaits for the claimed session or the typed rejection.
    pub fn claim_seat_session(
        name: SeatSessionName,
        owner: Owner,
    ) -> (
        Self,
        oneshot::Receiver<Result<SeatSession, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (Self::ClaimSeatSession { name, owner, reply }, rx)
    }

    /// Builds a `ReleaseSeatSession` command plus the receiver its
    /// caller awaits for the released session or the typed rejection.
    pub fn release_seat_session(
        name: SeatSessionName,
        owner: Owner,
    ) -> (
        Self,
        oneshot::Receiver<Result<SeatSession, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (Self::ReleaseSeatSession { name, owner, reply }, rx)
    }

    /// Builds an `AddSeat` command plus the receiver its caller awaits
    /// for the added seat or the typed rejection.
    pub fn add_seat(
        session: SeatSessionName,
        owner: Owner,
        name: SeatName,
        marker: Option<PatternId>,
        pose: Option<SeatPose>,
    ) -> (
        Self,
        oneshot::Receiver<Result<TableTopSeat, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (
            Self::AddSeat(Box::new(AddSeatRequest {
                session,
                owner,
                name,
                marker,
                pose,
                reply,
            })),
            rx,
        )
    }

    /// Builds an `UpdateSeat` command plus the receiver its caller
    /// awaits for the updated seat or the typed rejection.
    pub fn update_seat(
        seat: SeatId,
        owner: Owner,
        name: Option<SeatName>,
        marker: Option<Option<PatternId>>,
        pose: Option<SeatPose>,
    ) -> (
        Self,
        oneshot::Receiver<Result<TableTopSeat, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (
            Self::UpdateSeat(Box::new(UpdateSeatRequest {
                seat,
                owner,
                name,
                marker,
                pose,
                reply,
            })),
            rx,
        )
    }

    /// Builds a `RemoveSeat` command plus the receiver its caller
    /// awaits for the removed seat or the typed rejection.
    pub fn remove_seat(
        seat: SeatId,
        owner: Owner,
    ) -> (
        Self,
        oneshot::Receiver<Result<TableTopSeat, RegistrationError>>,
    ) {
        let (reply, rx) = oneshot::channel();
        (Self::RemoveSeat { seat, owner, reply }, rx)
    }
}
