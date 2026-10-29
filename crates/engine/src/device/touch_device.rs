use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::path::PathBuf;

use tangible_objects_model::ABS_MT_POSITION_X;
use tangible_objects_model::ABS_MT_POSITION_Y;
use tangible_objects_model::ABS_MT_SLOT;
use tangible_objects_model::ABS_MT_TRACKING_ID;
use tangible_objects_model::Calibration;
use tangible_objects_model::EventType;
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
    ///
    /// On `SYN_DROPPED` (the kernel's resync signal: the event queue
    /// overflowed and terminations may have been lost) the kernel MT
    /// slot state is queried via `EVIOCGMTSLOTS` and appended as an
    /// authoritative resync batch: the marker followed by every open
    /// slot's state and a `SYN_REPORT`. `TouchState` lifts every held
    /// contact absent from that batch at frame end - stale slots die
    /// where they were created, not by heuristic watchdog. When the
    /// ioctl fails no marker is emitted (stale view beats a wipe).
    pub fn fetch_events(&mut self) -> Result<Vec<RawEvent>, EngineError> {
        let mut events = Vec::new();
        let mut dropped = false;
        for event in self.device.fetch_events()? {
            let raw = RawEvent::from(&event);
            if raw.is_syn_dropped() {
                dropped = true;
                continue;
            }
            events.push(raw);
        }
        if dropped && let Ok(resync) = self.resync_events() {
            events.extend(resync);
        }
        Ok(events)
    }

    /// Builds the authoritative resync batch from kernel MT slot state:
    /// the `SYN_DROPPED` marker, `SLOT`/`TRACKING_ID`/`POSITION_X/Y`
    /// for every kernel-open contact, then `SYN_REPORT`.
    fn resync_events(&self) -> Result<Vec<RawEvent>, EngineError> {
        let abs = self.device.get_abs_state()?;
        let num_slots = abs[ABS_MT_SLOT.as_u16() as usize].maximum as usize + 1;
        let tracking = self.mt_slots(ABS_MT_TRACKING_ID.as_u16(), num_slots)?;
        let pos_x = self.mt_slots(ABS_MT_POSITION_X.as_u16(), num_slots)?;
        let pos_y = self.mt_slots(ABS_MT_POSITION_Y.as_u16(), num_slots)?;

        let mut events = vec![RawEvent::syn_dropped()];
        for slot in 0..num_slots {
            if tracking[slot] == -1 {
                continue;
            }
            events.extend([
                RawEvent::new(EventType::Abs, ABS_MT_SLOT, slot as i32),
                RawEvent::new(EventType::Abs, ABS_MT_TRACKING_ID, tracking[slot]),
                RawEvent::new(EventType::Abs, ABS_MT_POSITION_X, pos_x[slot]),
                RawEvent::new(EventType::Abs, ABS_MT_POSITION_Y, pos_y[slot]),
            ]);
        }
        events.push(RawEvent::syn_report());
        Ok(events)
    }

    /// `EVIOCGMTSLOTS(code)` ioctl: current kernel values of one
    /// `ABS_MT_*` axis across all slots - the resync source of truth.
    fn mt_slots(&self, code: u16, num_slots: usize) -> Result<Vec<i32>, EngineError> {
        let mut data = vec![0i32; num_slots + 1];
        data[0] = i32::from(code);
        // _IOC(_IOC_READ, 'E', 0x0a, 4 + 4 * num_slots)
        let size = 4 + 4 * num_slots;
        let request = 0x80000000_u64 | (u64::from(b'E') << 8) | 0x0A | ((size as u64) << 16);
        let rc = unsafe { libc::ioctl(self.device.as_raw_fd(), request, data.as_mut_ptr()) };
        if rc < 0 {
            return Err(EngineError::Io(std::io::Error::last_os_error()));
        }
        data.remove(0);
        Ok(data)
    }
}
