# Tracking Stabilization

Three mechanisms keep a tracked tangible stable across frames: slot
affinity, reserved slots and missing-point completion. Together they
prevent index churn, slot stealing and phantom re-compositions.

## Slot affinity

A `Tracker` binds the touch **slots** (`ABS_MT_SLOT`) of a matched
figure to its `TangibleId`. On subsequent frames the incumbent re-matches
on its *own* slots before the generic matcher runs - a slightly better
scoring composition can never steal the identity.

- Slot = stable identity across frames (touch indices are frame-local
  and meaningless)
- `TRACKING_ID = -1` ends the slot's binding immediately (real lift)
- bound slots live in `Tracker::slot_set`/`bound_slots` per `TangibleId`

## Reserved slots

Bound slots are excluded from the generic matching pool via
`find_matches_excluding` - even while the incumbent fails to match.
Reservation ends only when the tracked tangible expires
(`max_frames_without_match`, default 30) or the slot dies.

Result: a phantom made of 2 points of figure A + 1 of figure B cannot
form because B's slots are reserved for B's incumbent.

## Partial affinity

A 4+-pin figure that loses one contact may re-match as a *partial*
(`n-1`) on its bound slots - tracking survives transient pin loss.
Partials are excluded from suppression (suppressing the remaining pins
of a half-lifted figure would eat the lift).

## Partial spawns (`spawn_on_partial`)

A partial match may only *create* a tracked tangible when the profile
opts in (`spawn_on_partial = true` in the TOML, `spawn_onPartial` on the
register interfaces; default `false`). Incumbent re-matching is
unaffected - the flag only gates *birth* of a track, since a reduced
contact set can satisfy foreign partial fingerprints (phantom surface;
see `--similarity`).

A partial-spawned tangible starts **pending**: it reserves its slots but
emits no `TangibleAdded` until

- the partial match has held for `spawn_stability` consecutive frames
  (default 15, ~0.25 s), or
- an exact match confirms the same slot set immediately.

Pending tangibles appear in `tracked` with `pending = true` (recordings,
GraphQL, D-Bus, MCP), render dimmed in the GUI and suppress nothing.
A missed frame resets the counter; the expiry hysteresis is the normal
`frames_since_match` budget.

## Missing-point completion

When a partial match leaves `n-k` pins matched, the engine predicts each
missing pin's position from the similarity transform of the kept pins
and tries to **claim** an unreserved touch near the prediction.

- All consistent correspondences are enumerated; the transform with the
  lowest residual wins - symmetric figures (e.g. squares) do not get a
  mirrored prediction
- Claimed slots are recorded in `PatternMatch.claimed_indices`;
  `all_indices()` returns matched + claimed
- Claims never reuse already-claimed or reserved slots

## Completion configuration

`CompletionConfig` - `~/.config/tangible-objects/completion.toml` as
base, `--completion-*` flags override, live changes via GraphQL
(`completion`/`setCompletion`), MCP (`get`/`set_completion`) and D-Bus
(`completion`/`set_completion`):

```toml
radius_factor = 2.0   # claim radius = factor * pattern tolerance
max_missing = 1       # 1 = n-1 partials only; 2 = also n-2
persist = true        # claimed slots join the bound slot set
mode = "match"        # "match" | "exact"
```

| `mode` | Claimed slots |
|--------|---------------|
| `match` (default) | bound to the tangible, excluded from matching - but **not** suppressed |
| `exact` | a completed partial suppresses matched *and* claimed slots like an exact match |

## Motion distortion: smoothing + widened incumbent tolerance

Real panels report each contact with its own latency, so a moving
figure's measured geometry is *distorted* - single distances deviate
3-5 mm beyond the profile tolerance while the figure itself is rigid.
Two complementary countermeasures:

- **Per-slot position smoothing** (`--smoothing-alpha` /
  `smoothing_alpha` in `completion.toml`, default `0.6`, `0` = off):
  EMA over consecutive frames per slot. The differential latency is
  averaged out, deviations roughly halve; a new tracking ID on a slot
  restarts at its raw position. Costs ~0.5 frame of latency.
- **Widened incumbent tolerance** (`--affinity-factor` /
  `affinity_factor` in `completion.toml`, default `2.0`, `1.0` = off):
  the slot-bound re-match accepts `match_score <= factor` (per-index
  tolerances included, partials scale the same). Only incumbents
  benefit - new matches stay strict, so phantom risk does not grow.
  The slot binding already proves identity; distortion must not lose it.

Both knobs are part of `CompletionConfig` - the same runtime surface as
the completion settings: `completion`/`setCompletion` (GraphQL),
`get`/`set_completion` (MCP), `completion`/`set_completion` (D-Bus).

## Interaction

- **Regions** (`[[regions]]` in profiles) restrict where a pattern may
  match - combined with slot reservation, a neighboring figure's slots
  are doubly excluded (reserved + outside the region)
- **Suppression** (`--filter`) sees exact matches plus `exact`-mode
  completions; partials and `match`-mode claims stay visible
- **Expiry**: hysteresis (`max_frames_without_match`) is the only
  release path besides a real `TRACKING_ID = -1` lift

Not implemented (documented decision): global assignment/Hungarian
matching - incumbent-first + reservation removes the phantom problem
structurally.
