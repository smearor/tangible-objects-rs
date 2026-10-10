use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::path::PathBuf;

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::ABS_MT_POSITION_Y;
use tangible_objects_model::Calibration;
use tangible_objects_model::RawEvent;

use crate::EngineError;

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

/// An evdev multitouch input device.
///
/// Thin wrapper around [`evdev::Device`] that speaks the engine's
/// [`RawEvent`] vocabulary instead of `evdev::InputEvent`.
pub struct TouchDevice {
    device: evdev::Device,
}

impl TouchDevice {
    /// Opens an evdev device node (usually requires root or `input` group).
    pub fn open(path: &Path) -> Result<Self, EngineError> {
        let device = evdev::Device::open(path)?;
        Ok(Self { device })
    }

    /// Lists all `/dev/input/event*` devices that can be opened.
    pub fn enumerate() -> Vec<DeviceEntry> {
        let mut entries = Vec::new();
        for index in 0..32 {
            let path = PathBuf::from(format!("/dev/input/event{index}"));
            if let Ok(device) = evdev::Device::open(&path) {
                let name = device.name().unwrap_or("(unknown)").to_string();
                let is_multitouch = device
                    .supported_absolute_axes()
                    .map(|axes| axes.contains(evdev::AbsoluteAxisType::ABS_MT_SLOT))
                    .unwrap_or(false);
                entries.push(DeviceEntry {
                    path,
                    name,
                    is_multitouch,
                });
            }
        }
        entries
    }

    /// Device name reported by the kernel, if any.
    pub fn name(&self) -> Option<&str> {
        self.device.name()
    }

    /// Raw file descriptor of the device node (for `poll()`-based loops).
    pub fn as_raw_fd(&self) -> RawFd {
        self.device.as_raw_fd()
    }

    /// Reads the device calibration (`resolution` and min/max of the
    /// `ABS_MT_POSITION_X/Y` axes, units/mm).
    ///
    /// Returns `None` when the device reports no usable resolution.
    pub fn calibration(&self) -> Option<Calibration> {
        let abs_state = self.device.get_abs_state().ok()?;
        let x = &abs_state[ABS_MT_POSITION_X.as_u16() as usize];
        let y = &abs_state[ABS_MT_POSITION_Y.as_u16() as usize];
        let x_res = x.resolution as f64;
        let y_res = y.resolution as f64;
        if x_res > 0.0 && y_res > 0.0 {
            Some(Calibration::with_bounds(
                x_res,
                y_res,
                x.minimum as f64,
                x.maximum as f64,
                y.minimum as f64,
                y.maximum as f64,
            ))
        } else {
            None
        }
    }

    /// Fetches all pending input events converted to [`RawEvent`]s.
    pub fn fetch_events(&mut self) -> Result<Vec<RawEvent>, EngineError> {
        let mut events = Vec::new();
        for event in self.device.fetch_events()? {
            events.push(RawEvent::new(
                event.event_type().0,
                event.code(),
                event.value(),
            ));
        }
        Ok(events)
    }
}
