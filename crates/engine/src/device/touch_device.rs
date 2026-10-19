use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::path::PathBuf;

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::ABS_MT_POSITION_Y;
use tangible_objects_model::Calibration;
use tangible_objects_model::RawEvent;

use super::DeviceEntry;
use crate::EngineError;

/// An evdev multitouch input device.
///
/// Thin wrapper around [`evdev::Device`] that speaks the engine's
/// [`RawEvent`] vocabulary instead of `evdev::InputEvent`.
pub struct TouchDevice {
    /// The opened evdev device - source of raw events and target of
    /// `EVIOCGRAB`/`ungrab`/`EVIOCGABS` ioctls.
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
                    vendor: device.input_id().vendor(),
                    product: device.input_id().product(),
                    is_multitouch,
                });
            }
        }
        entries
    }

    /// Auto-detects the touchscreen: enumerates devices and matches their
    /// `vendor:product` pair against known [`DeviceProfile`]s. Succeeds
    /// only when exactly one known device is attached - zero matches ask
    /// for `--device`, several matches list the candidates.
    pub fn detect(
        registry: &tangible_objects_model::DeviceRegistry,
    ) -> Result<DeviceEntry, EngineError> {
        let mut matches: Vec<DeviceEntry> = Self::enumerate()
            .into_iter()
            .filter(|entry| registry.find_by_ids(entry.vendor, entry.product).is_some())
            .collect();
        match matches.len() {
            0 => Err(EngineError::InvalidInput(
                "no device with a known profile detected - pass --device /dev/input/eventX"
                    .to_string(),
            )),
            1 => Ok(matches.swap_remove(0)),
            _ => {
                let list = matches
                    .iter()
                    .map(|e| format!("{} ({e})", e.path.display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(EngineError::InvalidInput(format!(
                    "several known devices detected ({list}) - pass --device to disambiguate"
                )))
            }
        }
    }

    /// Device name reported by the kernel, if any.
    pub fn name(&self) -> Option<&str> {
        self.device.name()
    }

    /// Raw file descriptor of the device node (for `poll()`-based loops).
    pub fn as_raw_fd(&self) -> RawFd {
        self.device.as_raw_fd()
    }

    /// Exclusive grab (`EVIOCGRAB`): all other processes stop receiving
    /// events. Used by the filter proxy so tangible contacts stay invisible
    /// to the rest of the system.
    pub fn grab(&mut self) -> Result<(), EngineError> {
        self.device.grab()?;
        Ok(())
    }

    /// Read access to the wrapped evdev device - the filter proxy mirrors
    /// its capabilities onto a uinput clone.
    pub fn inner(&self) -> &evdev::Device {
        &self.device
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
            events.push(RawEvent::from(&event));
        }
        Ok(events)
    }
}
