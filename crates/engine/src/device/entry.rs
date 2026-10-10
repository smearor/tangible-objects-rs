use std::path::PathBuf;

/// An enumerated `/dev/input/eventX` entry.
#[derive(Debug)]
pub struct DeviceEntry {
    /// Device node path (e.g. `/dev/input/event3`).
    pub path: PathBuf,
    /// Device name reported by the kernel.
    pub name: String,
    /// Whether the device supports multitouch protocol type B
    /// (has the `ABS_MT_SLOT` axis).
    pub is_multitouch: bool,
}
