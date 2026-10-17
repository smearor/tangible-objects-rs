# FilterProxy: Filtering Semantics

Two independent mechanisms decide what reaches the virtual device:

1. **Slot filter** - multitouch events (`ABS_MT_*`, codes `0x2f..0x3e`)
   are slot-scoped; suppressed slots are dropped.
2. **Legacy filter** - `BTN_TOUCH`, `ABS_X`, `ABS_Y` are not slot-scoped
   (the driver emits them once per frame mirroring "a contact exists");
   they are buffered and decided at frame end.

## Slot filter

State per slot lives in three maps:

| Map | Meaning |
|-----|---------|
| `emitted` | slots with a live contact on the clone |
| `pending` | consecutive matched frames so far (gate counter) |
| `suppressed` | slots under suppression, mapped to frames since last match |
| `held` | withheld events of unclassified new contacts (Guarded) |

Rules per frame (executed in `finish`):

- **Graduation**: a slot matching `stability_gate` consecutive frames
  enters `suppressed` (default 2; `Immediate` skips the gate).
- **Hysteresis**: a suppressed slot survives 30 frames without a renewed
  match (`SUPPRESSION_HYSTERESIS`) - a flappy pin does not leak.
- **Release**: `TRACKING_ID = -1` on a suppressed slot ends suppression
  instantly; hysteresis expiry also releases.
- **Retraction**: a slot that already emitted a live contact and becomes
  suppressed gets a synthetic `SLOT` + `TRACKING_ID = -1` injection before
  the frame's events - the OS-level contact is closed.
- **Partial matches never suppress** (`PatternMatch::is_partial`) - the
  widened tolerance would eat finger touches resembling a profile.

## FilterMode policies

| Mode | Entry rule | Leak on placement | Phantom risk |
|------|-----------|-------------------|--------------|
| `off` | none (passthrough) | n/a | none |
| `immediate` | first matched frame | none | eats contacts on 1-frame phantoms |
| `gated` | `stability_gate` consecutive matches | ~1-2 frames | none |
| `guarded` (default) | gate + hold unclassified contacts | none | none |
| `tangibles-only` | n/a - all contacts suppressed | none | n/a |

`Guarded` detail: a fresh `TRACKING_ID >= 0` on a slot that is neither
emitted nor suppressed starts a hold; all that slot's events are buffered.
On the next frame: matched -> stays held (candidate); graduated ->
dropped; unmatched -> the buffered events are replayed and the contact
appears ~1 frame late but complete.

## Legacy filter (`LegacyMode`)

`BTN_TOUCH`, `ABS_X`, `ABS_Y` arrive after the slot events of a frame and
mirror the *merged* contact state - they cannot be attributed to a slot.

| Mode | Behaviour |
|------|-----------|
| `auto` (default) | emitted iff a contact was live at frame start **or** end |
| `always` | always forwarded |
| `never` | always dropped |

Why "start **or** end": the real evtest stream shows

```text
TRACKING_ID=-1  →  BTN_TOUCH=0  →  SYN_REPORT
```

The last contact's lift frame ends with an empty emitted set - if the
rule only checked frame end, `BTN_TOUCH=0` would be dropped and the clone
would stay stuck at `BTN_TOUCH=1` (a phantom key press). Emitting on the
down-transition keeps the clone's key state consistent; a `BTN_TOUCH=0`
on an idle device is harmless.

In `TangiblesOnly` mode retractions for still-live contacts are followed
by a synthetic `BTN_TOUCH=0` so no stale key state remains.

## CLI

```bash
--stability-frames <n>        # gate, default 2 (gated/guarded only)
--legacy <auto|always|never>  # legacy policy, default auto
--edge-filter-mm <mm>         # drop ghost contacts near panel borders
```
