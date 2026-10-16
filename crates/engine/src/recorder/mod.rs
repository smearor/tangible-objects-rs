//! Recording sinks for `--record` (TRAINING.md task 1.2).
//!
//! [`FrameRecorder`] writes one JSONL [`tangible_objects_model::FrameRecord`]
//! per non-empty [`crate::EngineFrame`]; [`RawRecorder`] writes every consumed
//! [`tangible_objects_model::RawEvent`] in `evtest`-compatible text format so
//! the artifact replays through [`crate::EvtestReplay`]. Both write on
//! dedicated threads behind bounded channels — a full buffer drops data with
//! a warning, it never stalls the device loop.

mod context;
mod frame;
pub(crate) mod raw;
mod session;

pub use context::FrameContext;
pub use frame::FrameRecorder;
pub use raw::RawRecorder;
pub use session::RecordingSession;
