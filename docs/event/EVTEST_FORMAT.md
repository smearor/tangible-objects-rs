# evtest Format

The `.evtest` text format used by the raw recording artifact
(`<base>.evtest`) and all `research/figure-tracking-*.txt` fixtures.

`evtest` is the Linux input-subsystem debugging tool
(`evtest /dev/input/eventX`): it dumps the device's capability header and
then every evdev event as human-readable text. The daemon *replays* this
format (`EvtestReplay`) and *writes* it (`RawRecorder`) — the
hardware-free round trip of the recording pipeline. Types are documented in
[`DATA_MODEL.md`](DATA_MODEL.md).

## File structure

```
Input driver version is 1.0.1
Input device ID: bus 0x3 vendor 0x457 product 0x11dc version 0x111
Input device name: "Silicon Integrated System Co. SiS HID Touch Controller"
Supported events:
  Event type 0 (EV_SYN)
  Event type 1 (EV_KEY)
    Event code 330 (BTN_TOUCH)
  Event type 3 (EV_ABS)
    Event code 53 (ABS_MT_POSITION_X)
      Value        0
      Min          0
      Max       4095
      Resolution   42
    Event code 54 (ABS_MT_POSITION_Y)
      ...
Testing ... (interrupt to exit)
Event: time 1787786579.073232, type 3 (EV_ABS), code 57 (ABS_MT_TRACKING_ID), value 40
Event: time 1787786579.073270, type 3 (EV_ABS), code 53 (ABS_MT_POSITION_X), value 2404
Event: time 1787786579.073270, type 3 (EV_ABS), code 54 (ABS_MT_POSITION_Y), value 3011
Event: time 1787786579.074730, -------------- SYN_REPORT ------------
Event: time 1787786579.074730, type 3 (EV_ABS), code 57 (ABS_MT_TRACKING_ID), value 41
...
```

### Header (consumed by `EvtestReplay::parse_header`)

| Line pattern                                    | Used for |
|-------------------------------------------------|----------|
| `Input device name: "…"`                        | `device_name()` — written back into recorded headers |
| `Event code N (ABS_MT_POSITION_X)` + `Value/Min/Max/Resolution` | X-axis calibration (`Calibration`) |
| `Event code N (ABS_MT_POSITION_Y)` + `Value/Min/Max/Resolution` | Y-axis calibration |
| `Testing ... (interrupt to exit)`               | Ends the header — event parsing starts here |

Other header lines (driver version, device ID, other event codes) are
skipped.

### Event lines

```
Event: time <sec>.<µsec>, type <n> (EV_XXX), code <n> (NAME), value <n>
Event: time <sec>.<µsec>, -------------- SYN_REPORT ------------
```

- `type`/`code`/`value` are parsed numerically; the parenthesized names are
  informational.
- The **delimiter form** marks a frame boundary (`SYN_REPORT`). The typed
  form `type 0 (EV_SYN), code 0 (SYN_REPORT), value 0` produced by some
  evtest variants parses to the same `RawEvent{Syn, 0, 0}` — both are frame
  boundaries (`RawEvent::is_syn_report` covers both).
- `SYN_DROPPED` delimiters are not `SYN_REPORT` and do not open a frame.
- `time` is seconds with microsecond precision; `next_event_timed` returns
  it so replay/recorded timestamps are deterministic.

## The code-number pitfall (recorded bug)

Event **code numbers are only unique per event type**: `code 0` is
`SYN_REPORT` under `EV_SYN` but `ABS_X` under `EV_ABS`. Writers must resolve
names type-aware — `NamedEventCode { event_type, code }` does this;
`EventCode` alone is intentionally not nameable. A type-ignorant writer once
emitted `code 0 (SYN_REPORT)` for an `EV_ABS` line and the parser created
phantom frames — the round-trip test (`--record` → `--replay`) caught it.
The parser additionally accepts `SYN_REPORT` delimiters only when the line
carries no `type` field.

## Producer / consumer

| Direction   | Implementation | Notes |
|-------------|----------------|-------|
| write       | `RawRecorder` (`crates/engine/src/recorder/raw.rs`) | Header (device name + `ABS_MT_POSITION_X/Y` blocks from `Calibration`), one event line per `RawEvent`, delimiter line per `SYN_REPORT` |
| read        | `EvtestReplay` (`crates/engine/src/replay.rs`) | Header → `Calibration` + `device_name`; lines → `RawEvent` (+ timestamp) |
| convert     | `DeviceConverter` (`crates/engine/src/convert`) | Re-emits a source artifact in a target `DeviceProfile`'s raw space — see [`../device/DATA_MODEL.md`](../device/DATA_MODEL.md) |
| canonical   | `evtest /dev/input/eventX` | Any real evtest dump is a valid fixture |

Determinism rule: replayed timestamps come from the file, so recording a
replay twice produces identical `.jsonl` output.
