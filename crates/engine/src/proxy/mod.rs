//! Filter proxy (FILTER_PROXY_VIRTUAL.md): the real evdev device is grabbed
//! exclusively and its events are re-emitted through a uinput clone
//! ([`VirtualTouchscreen`]) - minus the slots that a matched tangible
//! occupies ([`FilterProxy`]). The operating system sees the clone; tangible
//! contacts never reach it.

mod boundary;
mod filter;
mod legacy_mode;
mod stage;
mod virtual_device;

pub use filter::FilterProxy;
pub use legacy_mode::LegacyMode;
pub use virtual_device::VirtualTouchscreen;
