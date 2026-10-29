//! Multitouch contacts.
//!
//! [`TouchPoint`] is a single active contact tracked by its
//! [`TouchPointSlot`], the kernel-assigned slot index (protocol type B), and
//! its [`TrackingId`], the kernel-assigned contact lifetime identifier.

mod point;
mod slot;
mod slot_set;
mod touch_set;
mod tracking_id;

pub use point::TouchPoint;
pub use slot::TouchPointSlot;
pub use slot_set::SlotSet;
pub use touch_set::TouchSet;
pub use tracking_id::TrackingId;
