use tangible_objects_model::EventCount;

/// Statistics of one `.evtest` device conversion.
///
/// All fields count raw events; `EventCount` keeps them unconfusable with
/// frame counts.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConvertStats {
    /// Raw events read from the source artifact.
    pub events: EventCount,
    /// Events written to the target artifact.
    pub kept: EventCount,
    /// Position events whose value exceeded the target bounds and was
    /// clamped (source geometry outside the target surface).
    pub clamped: EventCount,
    /// Events dropped because the target device does not emit that
    /// `(type, code)` at all.
    pub dropped: EventCount,
}
