//! Collision matrix analysis (TRAINING.md 2.4).

mod accumulator;
mod cell;
mod report;

pub use cell::CollisionCell;
pub use report::CollisionReport;

pub(crate) use accumulator::Accumulator;
