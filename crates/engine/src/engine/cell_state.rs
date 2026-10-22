//! `CellState` - the debounce state of one occupancy field.

use tangible_objects_model::ContactCount;
use tangible_objects_model::FrameCount;

/// Per-field debounce state.
#[derive(Clone, Default)]
pub(crate) struct CellState {
    /// The last confirmed (emitted) contact count.
    pub stable_count: ContactCount,
    /// The candidate count currently being confirmed.
    pub pending_count: ContactCount,
    /// Consecutive frames at `pending_count`.
    pub run_length: FrameCount,
}
