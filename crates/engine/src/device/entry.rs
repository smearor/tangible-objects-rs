use std::fmt::Display;
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

impl Display for DeviceEntry {
    /// Renders as `<path>: <name>` plus a ` [Multitouch]` marker when the
    /// device speaks protocol type B.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.name)?;
        if self.is_multitouch {
            write!(f, " [Multitouch]")?;
        }
        Ok(())
    }
}
