//! Recording data model for the training and analysis pipeline.
//!
//! [`FrameRecord`] is the JSONL line written per `SYN_REPORT` frame by the
//! daemon's `FrameRecorder` (see `concepts/planned/TRAINING.md`): the frame's
//! [`crate::TouchPoint`]s, every tested [`CandidateReport`] (matched and
//! rejected) and the tracker view as [`TrackedRecord`]s, tagged with an
//! optional ground-truth [`RecordLabel`]. Record types are `Serialize`-only;
//! analysis happens offline in external tooling.

mod candidate;
mod frame;
mod frame_count;
mod frame_number;
mod label;
mod tracked;

pub use candidate::CandidateReport;
pub use frame::FrameRecord;
pub use frame::RECORD_SCHEMA_VERSION;
pub use frame_count::FrameCount;
pub use frame_number::FrameNumber;
pub use label::RecordLabel;
pub use tracked::TrackedRecord;
