//! Linux input event code constants (multitouch protocol type B).
//!
//! Event types are modeled by [`super::EventType`]; the constants below
//! name the event codes interpreted by the pipeline.

use super::EventCode;

/// Synchronization code delimiting one input frame.
pub const SYN_REPORT: EventCode = EventCode::new(0x00);

/// Absolute multitouch code selecting the active slot.
pub const ABS_MT_SLOT: EventCode = EventCode::new(47);

/// Absolute multitouch code for the contact X position.
pub const ABS_MT_POSITION_X: EventCode = EventCode::new(53);

/// Absolute multitouch code for the contact Y position.
pub const ABS_MT_POSITION_Y: EventCode = EventCode::new(54);

/// Absolute multitouch code for the contact tracking ID (-1 removes the slot).
pub const ABS_MT_TRACKING_ID: EventCode = EventCode::new(57);
