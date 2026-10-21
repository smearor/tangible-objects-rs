//! Daemon plumbing injected into the GUI's device loop - keeps this
//! crate free of daemon internals (`modes::control`, `modes::setup`).

use std::path::Path;

use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_engine::TouchDevice;
use tangible_objects_model::Calibration;
use tangible_objects_model::RecordingStatus;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::EngineRuntime;

/// Mode plumbing the embedding daemon supplies: the GUI owns the device
/// loop like the headless modes do, but control-command draining,
/// recording status and interface spawning stay the daemon's semantics -
/// injected as function pointers instead of a daemon dependency.
pub struct GuiHooks {
    /// Starts the interface servers (GraphQL/D-Bus/MCP) for the
    /// runtime's handle.
    pub spawn_interfaces: Box<dyn FnOnce(&EngineHandle) + Send>,
    /// Drains pending control commands into runtime/engine/recording -
    /// the daemon's shared `drain_control_commands`.
    pub drain_control_commands:
        fn(&mut EngineRuntime, &mut Option<RecordingSession>, &mut Engine, Option<&str>),
    /// Builds the `RecordingStatus` view published in snapshots.
    pub status_of_session: fn(&RecordingSession) -> RecordingStatus,
    /// Resolves the effective calibration for the opened device
    /// (device-profile override or the device's reported axes).
    pub effective_calibration: fn(&TouchDevice, Option<&Path>) -> Option<Calibration>,
    /// Applies a completed remote-training capture (`TrainProfile` →
    /// stable point set → `ProfileFile`) - typically the daemon's
    /// persistent `apply_registration` path.
    pub register_profile: fn(&mut EngineRuntime, &mut Engine, tangible_objects_model::ProfileFile),
}
