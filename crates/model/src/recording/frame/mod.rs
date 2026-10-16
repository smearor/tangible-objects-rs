//! Frame-level recording types: the `FrameRecord` line plus the three
//! frame-quantity newtypes.
//!
//! Deliberately distinct: [`FrameNumber`] is an ordinal ("frame 47"),
//! [`FrameCount`] a grow-only cardinality ("47 frames"), and
//! [`RemainingFrameCount`] a shrink-only budget ("47 frames left").

mod count;
mod number;
mod record;
mod remaining_count;

pub use count::FrameCount;
pub use number::FrameNumber;
pub use record::FrameRecord;
pub use record::RECORD_SCHEMA_VERSION;
pub use remaining_count::RemainingFrameCount;
