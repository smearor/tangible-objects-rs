use tangible_objects_engine::EngineEvent;
use tangible_objects_model::SetAction;

/// Unified domain event distributed to all interface crates.
///
/// Wraps engine-level events and Tangible Object Set actions so subscribers
/// need a single stream. Emission order per frame: engine events first,
/// then the set actions derived from the new tracked state.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Engine event: touch point or tangible changes.
    Engine(EngineEvent),
    /// Tangible Object Set action (TOS-ADD/REM/OBJ-*).
    Set(SetAction),
}
