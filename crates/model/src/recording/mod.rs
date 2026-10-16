//! Recording data model for the training and analysis pipeline.
//!
//! [`FrameRecord`] is the JSONL line written per `SYN_REPORT` frame by the
//! daemon's `FrameRecorder` (see `concepts/planned/TRAINING.md`): the frame's
//! [`crate::TouchPoint`]s, every tested [`CandidateReport`] (matched and
//! rejected) and the tracker view as [`TrackedRecord`]s, tagged with an
//! optional ground-truth [`RecordLabel`]. The same types deserialize in the
//! `analysis` crate for offline evaluation.

mod candidate;
mod frame;
mod label;
mod tracked;

pub use candidate::CandidateCount;
pub use candidate::CandidateReport;
pub use candidate::MatchScore;
pub use frame::FrameCount;
pub use frame::FrameNumber;
pub use frame::FrameRecord;
pub use frame::RECORD_SCHEMA_VERSION;
pub use frame::RemainingFrameCount;
pub use label::RecordLabel;
pub use tracked::TrackedRecord;
