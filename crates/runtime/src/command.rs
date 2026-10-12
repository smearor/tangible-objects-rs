/// Control command sent by an interface back to the daemon's device loop.
///
/// Commands are drained by the runtime after each published frame
/// ([`crate::EngineRuntime::try_recv_command`]); the concrete behavior is
/// implemented where the owning functionality lands (e.g. filter mode in
/// Phase 4, remote training with the interface crates).
#[derive(Debug, Clone)]
pub enum ControlCommand {
    /// Enable or disable the tangible filter mode.
    SetFilterMode(bool),
    /// Capture a new figure profile with the given name.
    TrainProfile(String),
}
