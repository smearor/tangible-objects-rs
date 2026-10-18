//! Filter proxy (FILTER_PROXY_VIRTUAL.md): the real evdev device is grabbed
//! exclusively and its events are re-emitted through a uinput clone
//! ([`VirtualTouchscreen`]) - minus the slots that a matched tangible
//! occupies ([`Pipeline`]). The operating system sees the clone; tangible
//! contacts never reach it.

mod boundary;
mod legacy_mode;
mod pipeline;
mod stage;
mod virtual_device;

pub use legacy_mode::LegacyMode;
pub use pipeline::Pipeline;
pub use virtual_device::VirtualTouchscreen;
