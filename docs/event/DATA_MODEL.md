# Event Data Model

The raw text wire format these types replay and record is documented in
[`EVTEST_FORMAT.md`](EVTEST_FORMAT.md).

Linux input event types - the module `model/src/event`. `RawEvent` is the
device-agnostic `(type, code, value)` triple mirroring the kernel's
`input_event`; it can be produced from `evdev` events or parsed from `evtest`
recordings without either dependency. `Engine::process` consumes one
`RawEvent` at a time; every `SYN_REPORT` delimits one frame.

```
crates/model/src/event/
├── mod.rs    - module docs + re-exports
├── type.rs   → EventType
├── code.rs   → EventCode
├── count.rs  → EventCount
├── named.rs  → NamedEventCode
├── input.rs  → SYN_REPORT, ABS_MT_* constants
└── raw.rs    → RawEvent
```

## EventType

Linux input event type - only the variants the pipeline interprets have named
representations; everything else maps to `Other(u16)` losslessly.

| Variant       | Raw value | Meaning |
|---------------|-----------|---------|
| `Syn`         | `0x00`    | `EV_SYN` - delimits one input frame |
| `Key`         | `0x01`    | `EV_KEY` - key/button events (e.g. `BTN_TOUCH`) |
| `Abs`         | `0x03`    | `EV_ABS` - absolute axes: slots, positions, tracking IDs |
| `Other(u16)`  | any       | Types not interpreted by the pipeline (preserved, not dropped) |

`From<u16>`/`From<EventType>` convert both ways; `Other` round-trips the
original numeric value.

## EventCode

Strongly typed event code within an `EventType` - `EventCode(pub u16)`. Plain
newtype (`new`/`as_u16`, `From` both ways, `Display`, `Ord`) so codes cannot
be confused with types or values. No serde: raw events are recorded in
`evtest` text format, not JSON (see
[TRAINING.md](../../concepts/planned/TRAINING.md)).

## Event code constants

The multitouch protocol type-B codes interpreted by `TouchState`, declared as
`EventCode` constants in `input.rs`:

| Constant             | Value | Role |
|----------------------|-------|------|
| `SYN_REPORT`         | `0x00` | Frame delimiter (`EventType::Syn`) |
| `ABS_MT_SLOT`        | `47`   | Selects the active slot |
| `ABS_MT_POSITION_X`  | `53`   | Contact X position (raw axis units) |
| `ABS_MT_POSITION_Y`  | `54`   | Contact Y position (raw axis units) |
| `ABS_MT_TRACKING_ID` | `57`   | Contact lifetime ID; `-1` releases the slot |

## RawEvent

One raw input event as `(type, code, value)` triple - `Copy`, `Eq`, no serde.

| Field        | Rust type    | Notes |
|--------------|--------------|-------|
| `event_type` | `EventType`  | `Syn` / `Key` / `Abs` / `Other` |
| `code`       | `EventCode`  | Code within the type |
| `value`      | `i32`        | Axis value, tracking ID or 0 |

`RawEvent::new(event_type, code, value)` accepts `impl Into<…>` so raw `u16`
codes/types convert implicitly. `is_syn_report()` is the frame boundary check
(`EventType::Syn && code == SYN_REPORT`).

## NamedEventCode & EventCount

| Type              | Wraps                          | Purpose |
|-------------------|--------------------------------|---------|
| `NamedEventCode`  | `(EventType, EventCode)`       | A code *paired with* its type - code namespaces are per event type (`code 0` = `SYN_REPORT` under `EV_SYN`, `ABS_X` under `EV_ABS`), so only the pair can be looked up for an `evtest`-style name |
| `EventCount`      | `u64`                          | Raw evdev event count - conversion stats, recording drop counters; distinct from `FrameCount` (SYN_REPORT-delimited frames) |

## Design Rules

- **Device-agnostic**: no `evdev` types leak into `model`; the `device` module
  in `engine` and the `EvtestReplay` parser both produce `RawEvent`s.
- **Lossless**: unknown types/codes survive as `Other`/raw numerics - a
  replayed or re-recorded stream stays byte-equivalent, which the
  `.evtest` raw artifact of a recording relies on.
- The frame boundary is the only semantic the engine itself consumes;
  everything else belongs to `TouchState`'s slot bookkeeping.
