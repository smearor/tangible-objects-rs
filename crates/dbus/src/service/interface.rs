//! `#[interface]` impl for [`EngineService`] - the D-Bus wire
//! surface. Method bodies delegate into the `crate::service::api`
//! topic modules; `#[zbus(signal)]` declarations stay inline (they
//! are part of the interface contract).
//!
//! zbus generates one `Interface` impl per `#[interface]` block, so
//! the methods of `org.tangibleobjects.Engine1` cannot be split
//! across impl blocks - delegation keeps the file readable instead.

use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::ProfileInfo;
use zbus::fdo;
use zbus::interface;
use zbus::message::Header;
use zbus::object_server::SignalEmitter;

use crate::service::api;
use crate::service::engine_service::EngineService;

#[interface(name = "org.tangibleobjects.Engine1")]
impl EngineService {
    /// All currently tracked tangibles.
    async fn list_tangibles(&self) -> Vec<TrackedTangible> {
        api::tangibles::list_tangibles(self).await
    }

    /// Loaded figure profiles.
    async fn list_profiles(&self) -> Vec<ProfileInfo> {
        api::profiles::list_profiles(self).await
    }

    /// Attached input device as a flat key-value list
    /// (`name`, `x_res`, `y_res`, `x_min`, `x_max`, `y_min`, `y_max`);
    /// empty values when no device/calibration is known.
    async fn device_info(&self) -> Vec<(String, String)> {
        api::device::device_info(self).await
    }

    /// Live recording status as a flat key-value list; empty while no
    /// session is or was active.
    async fn recording_status(&self) -> Vec<(String, String)> {
        api::recording::recording_status(self).await
    }

    /// Switches the filter proxy's suppression policy
    /// (`off`/`immediate`/`gated`/`guarded`); `stability > 0` overrides
    /// the consecutive-matched-frames gate, `0` keeps it. `stage`
    /// narrows the switch to that scope's tangible stage, `""` applies
    /// to all.
    async fn set_filter_mode(&self, mode: &str, stability: u64, stage: &str) -> fdo::Result<()> {
        api::filter::set_filter_mode(self, mode, stability, stage).await
    }

    /// Claims or creates a pipeline stage for `name` (EVENT_PIPELINE.md
    /// phase 7) - persists until `UnregisterStage`.
    async fn register_stage(&self, name: &str) -> fdo::Result<()> {
        api::stage::register_stage(self, name).await
    }

    /// Removes the pipeline stage for `name` - its suppression state
    /// dies with it.
    async fn unregister_stage(&self, name: &str) -> fdo::Result<()> {
        api::stage::unregister_stage(self, name).await
    }

    /// Starts a training session capturing a new profile with `name`.
    async fn train_profile(&self, name: &str) -> fdo::Result<()> {
        api::training::train_profile(self, name).await
    }

    /// Starts a recording session writing `<base_path>.jsonl` +
    /// `<base_path>.evtest`; an empty `label` records unlabeled.
    async fn start_recording(&self, base_path: &str, label: &str) -> fdo::Result<()> {
        api::recording::start_recording(self, base_path, label).await
    }

    /// Stops the active recording session.
    async fn stop_recording(&self) -> fdo::Result<()> {
        api::recording::stop_recording(self).await
    }

    /// Sets the ground-truth label for subsequent recorded frames; an empty
    /// string clears it.
    async fn set_record_label(&self, label: &str) -> fdo::Result<()> {
        api::recording::set_record_label(self, label).await
    }

    /// A new tangible was detected and is now tracked.
    #[zbus(signal)]
    pub async fn tangible_added(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible's pose changed.
    #[zbus(signal)]
    pub async fn tangible_moved(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A tracked tangible expired.
    #[zbus(signal)]
    pub async fn tangible_removed(
        emitter: &SignalEmitter<'_>,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// The set of active touch points changed.
    #[zbus(signal)]
    pub async fn touch_points_updated(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// All mandatory set members became present (TOS-ADD).
    #[zbus(signal)]
    pub async fn set_added(emitter: &SignalEmitter<'_>, set_id: TangibleSetId) -> zbus::Result<()>;

    /// The last mandatory set member disappeared (TOS-REM).
    #[zbus(signal)]
    pub async fn set_removed(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
    ) -> zbus::Result<()>;

    /// A member tangible appeared in an active set (TOS-OBJ-ADD).
    #[zbus(signal)]
    pub async fn object_added(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A member tangible disappeared (TOS-OBJ-REM).
    #[zbus(signal)]
    pub async fn object_removed(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// Active missing-pin completion configuration as a flat tuple
    /// (`radius_factor`, `max_missing`, `persist`, `mode`,
    /// `affinity_factor`, `smoothing_alpha`) - empty strings are
    /// impossible here; when nothing was published yet all values are
    /// defaults.
    async fn completion(&self) -> (f64, u32, bool, String, f64, f64) {
        api::completion::completion(self).await
    }

    /// Replaces the missing-pin completion configuration (`mode` is
    /// `match` or `exact`; `max_missing` must be 1 or 2; `affinity`
    /// widens incumbent tolerances, `smoothing` is the position EMA
    /// alpha).
    async fn set_completion(
        &self,
        radius_factor: f64,
        max_missing: u32,
        persist: bool,
        mode: &str,
        affinity: f64,
        smoothing: f64,
    ) -> fdo::Result<()> {
        api::completion::set_completion(
            self,
            radius_factor,
            max_missing,
            persist,
            mode,
            affinity,
            smoothing,
        )
        .await
    }

    /// Runs the deviation analysis over a recording artifact on the
    /// daemon host - same `mode`/`profiles_dir` semantics as the MCP
    /// `analyze_recording` tool (`"contacts"` re-evaluates every
    /// same-size contact subset of each frame).
    async fn analyze_recording(
        &self,
        path: &str,
        mode: &str,
        profiles_dir: &str,
    ) -> fdo::Result<String> {
        api::analysis::analyze_recording(self, path, mode, profiles_dir).await
    }

    /// Pairwise profile-collision analysis over a profile directory -
    /// every pair's minimum achievable max deviation over same-size
    /// contact subsets, classified as Safe/Partial/Exact risk.
    async fn analyze_similarity(&self, profiles_dir: &str) -> fdo::Result<String> {
        api::analysis::analyze_similarity(self, profiles_dir).await
    }

    /// Session metrics as JSON (uptime, re-ID churn, match quality,
    /// per-slot contact stability).
    async fn metrics(&self) -> String {
        api::metrics::metrics(self).await
    }

    /// Resets the session metric counters.
    async fn reset_metrics(&self) -> fdo::Result<()> {
        api::metrics::reset_metrics(self).await
    }

    /// Shows a user-facing message in the GUI overlay; an empty string
    /// clears it.
    async fn show_message(&self, text: &str) -> fdo::Result<()> {
        api::message::show_message(self, text).await
    }

    /// Shows or hides the daemon-embedded GUI window (`--gui` mode) -
    /// ignored on headless daemons.
    async fn set_gui_visible(&self, visible: bool) -> fdo::Result<()> {
        api::gui::set_gui_visible(self, visible).await
    }

    /// Registers a figure profile at runtime: `points` is the flat pin
    /// list `[x0, y0, x1, y1, ...]` in mm. `spawn_on_partial` allows
    /// partial spawns (SPAWN_AS_PARTIAL.md); `spawn_stability = 0` keeps
    /// the profile default. Queued to the device loop; fingerprint
    /// collisions are rejected there - verify via `list_profiles`.
    #[allow(clippy::too_many_arguments)]
    async fn register_pattern(
        &self,
        name: &str,
        points: Vec<f64>,
        tolerance: f64,
        owner: &str,
        persistent: bool,
        spawn_on_partial: bool,
        spawn_stability: u64,
        stage: &str,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        api::pattern::register_pattern(
            self,
            name,
            points,
            tolerance,
            owner,
            persistent,
            spawn_on_partial,
            spawn_stability,
            stage,
            &header,
        )
        .await
    }

    /// Replaces a registered pattern's geometry while keeping its ID.
    #[allow(clippy::too_many_arguments)]
    async fn update_profile(
        &self,
        id: u32,
        name: &str,
        points: Vec<f64>,
        tolerance: f64,
        spawn_on_partial: bool,
        spawn_stability: u64,
        stage: &str,
    ) -> fdo::Result<()> {
        api::pattern::update_profile(
            self,
            id,
            name,
            points,
            tolerance,
            spawn_on_partial,
            spawn_stability,
            stage,
        )
        .await
    }

    /// Removes a dynamically registered pattern by ID.
    async fn unregister_pattern(&self, id: u32) -> fdo::Result<()> {
        api::pattern::unregister_pattern(self, id).await
    }

    /// Registers an occupancy field set from a TOML string (same shape as
    /// `--fields` files).
    async fn register_field_set(
        &self,
        toml_str: &str,
        owner: &str,
        persistent: bool,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        api::field_set::register_field_set(self, toml_str, owner, persistent, &header).await
    }

    /// Removes a field set by name.
    async fn unregister_field_set(&self, name: &str) -> fdo::Result<()> {
        api::field_set::unregister_field_set(self, name).await
    }

    /// Removes every pattern and field set owned by `owner` - app
    /// cleanup hook; automatic cleanup also fires when a D-Bus caller's
    /// unique name vanishes.
    async fn unregister_owner(&self, owner: &str) -> fdo::Result<()> {
        api::owner::unregister_owner(self, owner).await
    }

    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    #[zbus(signal)]
    pub async fn object_moved(
        emitter: &SignalEmitter<'_>,
        set_id: TangibleSetId,
        tangible: &TrackedTangible,
    ) -> zbus::Result<()>;

    /// A field became occupied (debounced placed transition).
    #[zbus(signal)]
    pub async fn field_placed(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
    ) -> zbus::Result<()>;

    /// A field became empty (debounced lifted transition).
    #[zbus(signal)]
    pub async fn field_lifted(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
    ) -> zbus::Result<()>;

    /// A field's debounced contact count changed.
    #[zbus(signal)]
    pub async fn field_count_changed(
        emitter: &SignalEmitter<'_>,
        set: &str,
        field: &str,
        count: u32,
    ) -> zbus::Result<()>;
}
