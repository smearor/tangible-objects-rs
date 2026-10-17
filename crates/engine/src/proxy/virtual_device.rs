use evdev::AbsInfo;
use evdev::AttributeSet;
use evdev::InputEvent;
use evdev::PropType;
use evdev::UinputAbsSetup;
use evdev::uinput::VirtualDevice;
use evdev::uinput::VirtualDeviceBuilder;

use crate::EngineError;

/// A uinput clone of the real touchscreen.
///
/// Mirrors the source device's name, input IDs, properties, keys, misc
/// events and absolute axes so compositors and libinput treat it like the
/// original. `INPUT_PROP_DIRECT` is forced on - without it the clone would
/// be classified as a touchpad.
pub struct VirtualTouchscreen {
    /// The writable uinput device.
    device: VirtualDevice,
}

impl VirtualTouchscreen {
    /// Creates a virtual device mirroring `source`'s capabilities.
    pub fn create(source: &evdev::Device) -> Result<Self, EngineError> {
        let mut properties = AttributeSet::<PropType>::from_iter(source.properties().iter());
        properties.insert(PropType::DIRECT);

        let mut builder = VirtualDeviceBuilder::new()?
            .name(source.name().unwrap_or("tangible-objects touchscreen"))
            .input_id(source.input_id())
            .with_properties(&properties)?;

        if let Some(keys) = source.supported_keys() {
            builder = builder.with_keys(keys)?;
        }
        if let Some(misc) = source.misc_properties() {
            builder = builder.with_msc(misc)?;
        }
        if let Some(axes) = source.supported_absolute_axes() {
            let state = source.get_abs_state()?;
            for axis in axes.iter() {
                let info = &state[usize::from(axis.0)];
                let setup = UinputAbsSetup::new(
                    axis,
                    AbsInfo::new(
                        info.value,
                        info.minimum,
                        info.maximum,
                        info.fuzz,
                        info.flat,
                        info.resolution,
                    ),
                );
                builder = builder.with_absolute_axis(&setup)?;
            }
        }

        Ok(Self {
            device: builder.build()?,
        })
    }

    /// Emits a batch of events on the virtual device.
    ///
    /// Timestamps are assigned by `InputEvent::new` at emission time - the
    /// proxy adds at most one frame of latency, so source timestamps are not
    /// carried over.
    pub fn emit(&mut self, events: &[InputEvent]) -> Result<(), EngineError> {
        self.device.emit(events)?;
        Ok(())
    }
}
