//! evdev multitouch input devices.
//!
//! [`TouchDevice`] wraps an opened `/dev/input/eventX` node; [`DeviceEntry`]
//! describes one enumerated device.

mod entry;
mod touch_device;

pub use entry::DeviceEntry;
pub use touch_device::TouchDevice;
