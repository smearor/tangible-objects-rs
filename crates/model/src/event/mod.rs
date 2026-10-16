//! Linux input events: classification, code constants and the raw triple.
//!
//! [`EventType`] classifies raw input events, [`EventCode`] identifies the
//! code within a type, the constants name the codes interpreted by the
//! pipeline, and [`RawEvent`] is the device-agnostic `(type, code, value)`
//! triple.

mod code;
mod input;
mod named;
mod raw;
mod r#type;

pub use code::EventCode;
pub use input::ABS_MT_POSITION_X;
pub use input::ABS_MT_POSITION_Y;
pub use input::ABS_MT_SLOT;
pub use input::ABS_MT_TRACKING_ID;
pub use input::SYN_REPORT;
pub use named::NamedEventCode;
pub use raw::RawEvent;
pub use r#type::EventType;
