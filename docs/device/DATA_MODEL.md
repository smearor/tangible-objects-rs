# Device Data Model

Touchscreen device profiles and related record types - the module
`model/src/device` plus the conversion types in `engine/src/convert`.
Runtime behavior (registry loading, auto-detection, the conversion
pipeline, remote access) is documented separately in
[`DETECTION.md`](DETECTION.md) and [`CONVERSION.md`](CONVERSION.md).

```
crates/model/src/
├── devices/                 - compiled-in profiles (*.toml, include_str!)
└── device/
    ├── mod.rs               - module docs + re-exports
    ├── profile.rs           → DeviceProfile
    ├── registry.rs          → DeviceRegistry
    └── error.rs             → DeviceError

crates/engine/src/
├── device/
│   ├── entry.rs             → DeviceEntry
│   └── touch_device.rs      → TouchDevice
└── convert/
    ├── mod.rs
    ├── converter.rs         → DeviceConverter
    └── stats.rs             → ConvertStats
```

## DeviceProfile

TOML-serializable device profile - the persistent form of a touchscreen's
capabilities. Bundles the affine axis map (calibration raw material) and
the emitted event-code set.

| Field            | Rust type              | TOML             | Notes |
|------------------|------------------------|------------------|-------|
| `name`           | `String`               | `"SiS …"`        | `Input device name` header line - registry lookup key |
| `vendor`         | `Option<u16>`          | `0x457`          | evdev vendor ID for auto-matching |
| `product`        | `Option<u16>`          | `0x11dc`         | evdev product ID |
| `x_res`/`y_res`  | `UnitsPerMm`           | `7.0`/`12.0`     | Units per mm (`Resolution` of `ABS_MT_POSITION_X/Y`) |
| `x_min`/`x_max`  | `RawAxisValue`         | `0`/`4095`       | Raw axis bounds |
| `y_min`/`y_max`  | `RawAxisValue`         | `0`/`4095`       | Raw axis bounds |
| `abs_codes`      | `Vec<u16>`             | `[53, 54, 57]`   | `EV_ABS` codes the device emits - conversion drops anything unlisted |
| `key_codes`      | `Vec<u16>`             | `[330]`          | `EV_KEY` codes (e.g. `BTN_TOUCH`) |
| `report_rate_hz` | `Option<u16>`          | `120`            | Typical cadence - informational; conversion keeps source timestamps |

### Methods

| Method         | Signature                                          | Purpose |
|----------------|----------------------------------------------------|---------|
| `calibration`  | `(&self) -> Calibration`                           | The affine axis map (`raw / res → mm`) |
| `supports`     | `(EventType, EventCode) -> bool`                   | Capability check - `EV_SYN` is always supported (delimiter, not capability) |
| `to_raw`       | `(Millimeters, axis_x) -> (RawAxisValue, bool)`    | Inverse of `Calibration::to_mm`, clamped to `[min, max]`; the flag reports clamping |

## DeviceRegistry

Collection of known profiles - builtins first, then directory-loaded ones;
load failures collect into `errors` like `ProfileDir`.

| Field      | Rust type                        | Notes |
|------------|----------------------------------|-------|
| `profiles` | `Vec<DeviceProfile>`             | All registered profiles |
| `errors`   | `Vec<(PathBuf, DeviceError)>`    | Files that failed to load |

See [`DETECTION.md`](DETECTION.md) for loading order and lookup rules.

## DeviceEntry

One enumerated `/dev/input/eventX` node.

| Field           | Rust type  | Notes |
|-----------------|------------|-------|
| `path`          | `PathBuf`  | Device node (`/dev/input/event3`) |
| `name`          | `String`   | Kernel-reported device name |
| `vendor`        | `u16`      | `InputId::vendor` - `vendor 0x…` in evtest headers |
| `product`       | `u16`      | `InputId::product` |
| `is_multitouch` | `bool`     | Has the `ABS_MT_SLOT` axis (protocol type B) |

`Display` renders `<path>: <name> [vendor:product] [Multitouch]`.

## ConvertStats

Counters of one `.evtest` device conversion - every field is an
[`EventCount`](../recording/DATA_MODEL.md) of raw evdev events.

| Field     | Notes |
|-----------|-------|
| `events`  | Raw events read from the source artifact |
| `kept`    | Events written to the target artifact |
| `clamped` | Position events clamped to target bounds (source geometry outside the target surface) |
| `dropped` | Events whose `(type, code)` the target does not emit |

## DeviceError

| Variant | Source           | Meaning |
|---------|------------------|---------|
| `Io`    | `std::io::Error` | Reading a profile file failed (carries path) |
| `Parse` | `toml::de::Error`| File did not parse into a `DeviceProfile` |

## Runtime snapshot types

`EngineSnapshot` carries device-facing state for the interfaces:

| Type              | Fields | Notes |
|-------------------|--------|-------|
| `DeviceInfo`      | `name` + `Option<UnitsPerMm>` (`x_res`/`y_res`) and `Option<RawAxisValue>` (`x_min`…`y_max`) | Set once per session; GraphQL `SimpleObject`, MCP `Serialize` - same unit types as `Calibration`, so raw axis values cannot be misread as millimeters |
| `RecordingStatus` | `active`, `label: Option<RecordLabel>`, `recorded_frames`/`dropped_frames: FrameCount`, `dropped_events: EventCount` | Updated per frame; `None` = no session |
