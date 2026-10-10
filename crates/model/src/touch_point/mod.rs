//! Multitouch contacts.
//!
//! [`TouchPoint`] is a single active contact tracked by its
//! [`TouchPointSlot`], the kernel-assigned slot index (protocol type B).

mod point;
mod slot;

pub use point::TouchPoint;
pub use slot::TouchPointSlot;
