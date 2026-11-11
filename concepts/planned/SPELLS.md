# Spells - Multi-Finger Gesture Recognition Framework

**Status:** Concept. First step: recognition + debug visualization +
training; action dispatch comes later.

**Reference:** [Hexecute](https://github.com/m31-galaxy/Hexecute) - a
gesture launcher that captures the whole screen on a keybind and
recognizes a single drawn figure. Our problem is harder: spells must
be detected **in-line** in the continuous touch stream, with no
explicit activation, alongside tangibles and normal pass-through.

## Goal & Motivation

A *spell* is a drawn gesture: the user strokes a shape with one or
more fingers to trigger an action. Where hexecute launches programs,
robofigure's spells are a **framework primitive** like field sets -
the daemon detects them; applications decide what they mean
(app launcher, menus, Hyprland workspace switch, game commands).

Requirements from the feature request:

- Per-finger polyline with a matching corridor.
- Finger count configurable per spell.
- Collision-avoidance with tangible detection via ≥2 fingers.
- Fields are zones where a spell cannot execute.
- Gesture slots tracked like tangible slots.
- Per-finger separate polylines (better than one merged stroke).
- A spell's contacts are NOT forwarded to the virtual device
  (suppressed like tangibles).
- Global spells in config; apps register/unregister at runtime.
- Spells are trainable.
- **Phase model**: explicit Einläutung/Arming per spell
  (exclusive zone, start-spot, five-finger-tap, tangible-chord),
  then Durchführung (trajectory) and Beendigung (dispatch) -
  suppression is state-gated by the armed phase, not by match
  probability.

## Terminology & Data Model

```toml
# ~/.config/tangible-objects/spells.d/pentagram.toml (or runtime-registered)
name = "Pentagram"
fingers = 2                 # FingerCount - simultaneous contacts
corridor = 12.0             # Millimeters - max deviation per stroke
scale_invariant = true      # gesture may be drawn larger/smaller
enabled = true
arming = "exclusive-zone"   # SpellArming - see Lifecycle

# one entry per finger - each is the *template* polyline for that
# finger in pattern space (mm, normalized: origin at first point,
# uniformly scaled to a reference extent when scale_invariant)
[[strokes]]
points = [[0.0, 0.0], [30.0, 40.0], [60.0, 0.0]]
[[strokes]]
points = [[0.0, 60.0], [60.0, 60.0]]
```

- `SpellName` - newtype, addresses the spell for unregister/store.
- `FingerCount` - newtype (`1..=10`).
- `StrokePolyline` - `Vec<Vec2<Millimeters>>`, one per finger.
- `Spell` - the above; serializes from `spells.d/*.toml` exactly like
  profiles (`ProfileFile` pattern: `load`, `load_dir`, `save`).
- `SpellMatched` - `LifecycleEvent` variant carrying name + surface
  center + matched score.
- `SpellArming` - how a spell's gesture is initiated (the
  discrimination mechanism - see Lifecycle):
  - `exclusive-zone` - a reserved surface region; contacts inside it
    are spell-dedicated
  - `start-spot` - a small region (screen corner, window edge,
    button) that arms the spell when touched first
  - `five-finger-tap` - a brief 5-contact tap (the whole hand)
    arms a short window; the gesture itself may use fewer fingers
  - `tangible-chord` - a registered tangible acts as the wand:
    armed while the tangible is tracked, its center pose is the
    stroke, lift completes
  Every spell declares its own arming type.

## Recognition

### Trajectory accumulation

`GestureTracker` (engine) watches the same event stream the touch
state consumes: per slot it accumulates `(pos, timestamp)` from
`ABS_MT_TRACKING_ID`-down until lift. A slot's trajectory is the
finger's stroke. Trajectories are retained briefly after lift so a
frame at "all fingers up" sees the complete stroke set.

### Trajectory normalization

Before matching, each trajectory is normalized like the template:

1. **Translate** - first point → origin (spell is position-free:
   drawable anywhere on the surface).
2. **Scale** (if `scale_invariant`) - uniform scale so the stroke's
   reference extent (max pairwise distance) matches the template's.
   Corridors are then compared in pattern space.
3. **Resample** - arc-length resample to N=64 points (kills sampling
   density noise, like the $-family recognizers).
4. Rotation is **not** normalized - direction matters for action
   semantics ("circle up" vs "circle down" can be different spells).

### Per-stroke matching: corridor via discrete Fréchet

A trajectory fits a template stroke when their **discrete Fréchet
distance ≤ corridor**. Fréchet is the right primitive for "stay
inside a corridor with monotonic progress" - it tolerates speed
variation (sampling points don't need to align index-wise) while
still enforcing direction and shape.

### Multi-finger assignment

With `fingers = k`, the engine tries every assignment of the k
trajectories to the k template strokes (k! permutations, k ≤ ~5 →
≤120 trivial evaluations) and takes the minimal total deviation.
Score = max per-stroke Fréchet (a spell only matches when *every*
finger stays in its corridor).

## Lifecycle: Einläutung → Durchführung → Beendigung

The discrimination problem (spell vs tangible vs pass-through,
no hotkey) resolves structurally once spells are **explicitly
armed**: suppression is state-gated, not match-gated. The gesture
tracker is a small state machine:

```
Idle/Listening → (arming trigger) → Armed → (stroke starts) → Active
     ↑                                          |
     ← (timeout, no stroke) ←————————————————————┘
Active → (all contacts up / wand lifted / left zone) → Dispatched
Dispatched → match → SpellMatched + consumed, or abort → Idle
```

**Suppression by phase:** in `Listening` every contact flows
unfiltered. In `Armed`/`Active` the spell's contacts are routed to
the gesture tracker and suppressed from forwarding - no match
probability needed, the armed state is the justification. In
`Dispatched` the stroke set is either consumed (match) or, for
arming types where it makes sense, replayed (see defer-and-replay
note for free-form arming below).

### Arming types

**`exclusive-zone`** - a spell-dedicated surface region
(`SpellZone`, same rectangle/polygon form as field regions):

- No field may overlap an exclusive zone (load-time validation).
- Tangible **matching is disabled inside the zone** (zone points are
  removed from the matching pool) - but *already-tracked* tangibles
  may be dragged through: their slots stay bound to the incumbent,
  so incumbent re-match keeps working and a wand-tangible crossing
  the zone neither spawns a phantom nor loses its track.
- Contacts inside the zone are always gesture candidates and never
  forward - zone semantics, zero ambiguity, zero deferral.
- Zones register like field sets (`exclusive_zones.d` or per-spell
  region), so apps can carve UI panels/toolbars out of the surface.

**`start-spot`** - a small region that arms the spell when the
first contact lands on it (screen corner, window edge, a button an
application registered). Same suppression story as the zone, just
arm-then-draw: the arming contact itself is the gesture's first
stroke point. Ideal for app-scoped spells - the spot binds the
gesture to the app's context.

**`five-finger-tap`** - a brief simultaneous contact of a whole
hand (≈5 contacts within a tight time window, e.g. Δt < 150 ms,
total dwell < ~300 ms):

- The tap only *arms*; the following stroke may use fewer fingers
  (1-3) - activation effort is cleanly separated from drawing.
- The tap constellation itself is a 5-point figure - it can be
  recorded as a pattern-like **handprint** (`HandProfile`), so
  different people's hands could arm different spell sets (per-user
  spells, a natural fit for the existing pattern matcher).
- The armed window is short (~1-2 s); contacts during the window
  are gesture candidates and suppress - residual ambiguity is tiny
  because the user just stamped intent.
- A five-finger pattern is extremely unlikely to collide with
  normal usage or tangible geometries.

**`tangible-chord`** - a registered tangible becomes the wand:

- Arming = the tangible being tracked (possibly a designated
  "wand" profile); Durchführung = the tangible's pose trajectory
  (center path, optionally rotation); Beendigung = lifting the
  object.
- The wand's slots were already suppressed (tangible-bound), so
  chording adds **zero new ambiguity** - it's the cheapest arming
  to implement and the most magical UX (cast spells with a figure).
- Stroke = the wand's center trajectory; a single-finger spell
  shape applies to the wand's center path.
- Chording can *compose* with tap/zone arming (wand inside a zone,
  wand after hand-tap) but each spell declares one primary type.

### Field exclusion (orthogonal rule)

Fields are zones where a spell cannot execute: a stroke whose
first contact lands inside an enabled field is never a gesture
candidate, whatever the arming type. Fields own their zones for
application semantics.

### Free-form arming (optional fallback)

If a spell wants no explicit trigger (`arming = "free"` - probably
only for ≥3-finger gestures where ambiguity is minimal), the
prefix-gated defer-and-replay mechanism applies: candidates buffer
only while some spell can still match, flush on divergence (see
the pinch/rotate note in Open Points). Armed modes never defer.

## Slot Tracking

Gesture slots get tracked like tangible slots: the gesture tracker
keeps slot → trajectory state across frames, survives the same
EMA/dropout handling (`smooth_touches` output is the source), and a
matched spell's slots become `spell_bound` - they cannot be claimed
by tangible matching afterwards (matches skip claimed contacts
already - the gesture tracker feeds the same `Claim` machinery or
reports slots via `bound_slots`). During `Armed`/`Active` phases
the gesture slots are removed from the matching pool so a moving
finger constellation can't spawn a phantom tangible.

## Interfaces & Registration

- `Registration::Spell(Spell)` - same `Register`/`Unregister` path
  with `owner` + `lifetime` as profiles/field-sets/dead-zones
  (`spells.d` store for persistent).
- `Registration::SpellZone(region)` / `StartSpot` / `HandProfile` -
  the arming primitives register the same way (zones and spots are
  region-like resources with owners; handprints are 5-point
  pattern-like profiles).
- `ControlCommand::{RegisterSpell, UnregisterSpell}` - or reuse the
  generic `Register` variant.
- `LifecycleEvent::SpellMatched { name, center, score }` - apps
  subscribe (GraphQL/D-Bus/MCP) to map spells to actions - the
  daemon framework ends here; dispatch is the app's business.
- Global spells: `spells.d/*.toml` next to `profiles/` - loaded in
  `setup.rs` like `load_dead_zones`.

## Debug Visualization

- **Live trajectories** (GUI): in-flight gesture candidates drawn
  as fading polylines in a distinct spell color; completed+matched
  strokes flash the spell name as a radial label.
- `spells_visible` runtime flag + `SetSpellsVisible` command +
  viz-card toggle (the dead-zones-visualization blueprint covers
  every plumbing layer).
- **Spell debug view**: per-stroke Fréchet score vs corridor,
  per-finger assignment - a dev aid like `profile_bounds`.

## Training

A `spell training` mode (mirroring figure `Training`):

- `GuiMode::SpellTraining` (or a third mode enum variant) -
  the user draws the gesture; on "all fingers up" the completed
  trajectories are resampled/normalized and offered as the spell's
  template strokes.
- hexecute convention: **draw 3×** - the trainer stores the attempt
  set; the template = the attempt minimizing average deviation, or
  the merged/averaged polyline. Corridor auto-suggestion = p99 of
  attempt deviations + margin (same idea as `tolerances` recording).
- Saved to `spells.d/<name>.toml` like profiles to `profiles/`.
- The training card gains the spell flow (name entry + save), or a
  dedicated spell-training card - GUI.md's card model already gives
  the pattern.

## Open Points / Research Notes

- **Other references** (beyond hexecute):
  - **$1 Unistroke Recognizer** (Wobbrock 2007) and successors
    **$P** (point-cloud, rotation-invariant, no alignment) and
    **$N** (multi-stroke, sequential) - the academic baseline;
    ~100 LOC each. $N handles sequential strokes, NOT simultaneous
    multi-finger - which is why we do per-finger trajectories +
    assignment instead.
  - **Protractor** - $1 variant using cosine similarity.
  - **touchegg / libinput-gestures / fusuma** - Linux multi-touch
    daemons, directional swipes/pinches only (no drawn figures).
  - **easystroke** - X11 mouse-gesture daemon (drawn shapes).
  - Hexecute itself: single-stroke shapes, learn-by-repetition
    (draw 3×), `gestures.json` store.
- **Pinch/rotate caveat** - the phase model mostly dissolves it:
  armed spells suppress without ambiguity, so pinch/rotate is only
  at risk under `arming = "free"` (the optional fallback), where
  the prefix gate keeps deferral similarity-proportional plus the
  `defer_budget_ms` cap bounds it. A possible further early-out:
  pinch/rotate has high inter-finger-distance change at low
  per-finger path length - a movement-signature classifier could
  veto free-form candidacy before buffering starts.
- **Five-finger-tap details**: the tap window/dwell constants and
  whether the arming contacts themselves suppress (they've lifted
  before the armed window - nothing to replay) need tuning on the
  physical device.
- **Wand trajectory**: for `tangible-chord`, whether the stroke is
  the wand's center path only or also its rotation (a drawn arc +
  twist semantics) is a per-spell option.
- Corridor matching chosen over $-family statistics: deterministic,
  debuggable per-stroke, honors the "Korridor" mental model, and
  composes cleanly with the per-finger assignment. $P remains a
  fallback if corridor matching proves too brittle for freehand.
- Single-finger spells stay possible (`fingers = 1`) but conflict
  with normal pointer drags - the defer-and-replay budget applies;
  recommended deployments register ≥2-finger spells.
- Rotation invariance intentionally off by default (direction
  semantics); could become a per-spell flag later.
