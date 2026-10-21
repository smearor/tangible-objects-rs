//! Touchscreen device profiles.
//!
//! A [`DeviceProfile`] bundles everything needed to convert raw evdev
//! recordings between devices: the affine axis map ([`Calibration`]), the
//! emitted event-code set, and the physical report characteristics. Known
//! devices are compiled into the binary via `devices/*.toml`; unknown ones
//! load from a directory (`--device-profiles`).

mod error;
mod info;
mod profile;
mod registry;

pub use error::DeviceError;
pub use info::DeviceInfo;
pub use profile::DeviceProfile;
pub use registry::DeviceRegistry;
