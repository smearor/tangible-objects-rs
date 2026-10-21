use super::engine_event::EngineEvent;
use super::lifecycle_event::LifecycleEvent;
use crate::SetAction;

/// Unified domain event distributed to all interface crates.
///
/// Wraps engine-level events, Tangible Object Set actions and lifecycle
/// events so subscribers need a single stream. Emission order per frame:
/// engine events first, then the set actions derived from the new tracked
/// state; lifecycle events arrive whenever a control command applies.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Engine event: touch point or tangible changes.
    Engine(EngineEvent),
    /// Tangible Object Set action (TOS-ADD/REM/OBJ-*).
    Set(SetAction),
    /// Lifecycle event: an applied control command (registration, config
    /// change, recording state).
    Lifecycle(LifecycleEvent),
}
