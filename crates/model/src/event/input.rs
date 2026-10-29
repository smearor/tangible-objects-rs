//! Linux input event code constants (multitouch protocol type B).
//!
//! Event types are modeled by [`super::EventType`]; the constants below
//! name the event codes interpreted by the pipeline.

use super::EventCode;

/// Synchronization code delimiting one input frame.
pub const SYN_REPORT: EventCode = EventCode::new(0x00);

/// `SYN_DROPPED` - the kernel's resync signal: the reader's event queue
/// overflowed, events were dropped and the client's accumulated state
/// may diverge from kernel truth (STALE_SLOTS root cause). The correct
/// client response is an `EVIOCGMTSLOTS` resync, never heuristics.
pub const SYN_DROPPED: EventCode = EventCode::new(0x03);

/// Absolute multitouch code selecting the active slot.
pub const ABS_MT_SLOT: EventCode = EventCode::new(47);

/// Absolute multitouch code for the contact X position.
pub const ABS_MT_POSITION_X: EventCode = EventCode::new(53);

/// Absolute multitouch code for the contact Y position.
pub const ABS_MT_POSITION_Y: EventCode = EventCode::new(54);

/// Absolute multitouch code for the contact tracking ID (-1 removes the slot).
pub const ABS_MT_TRACKING_ID: EventCode = EventCode::new(57);

/// Legacy single-touch axis: X position of the (first) contact.
/// Not slot-scoped - mirrors whether any contact is active.
pub const ABS_X: EventCode = EventCode::new(0);

/// Legacy single-touch axis: Y position of the (first) contact.
pub const ABS_Y: EventCode = EventCode::new(1);

/// Legacy single-touch key: pressed while at least one contact exists.
/// Not slot-scoped.
pub const BTN_TOUCH: EventCode = EventCode::new(330);
