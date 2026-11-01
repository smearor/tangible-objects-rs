# Dynamic Profile Registration

Applications register figure profiles and occupancy field sets at
runtime instead of shipping files in `profiles/`. Registrations carry an
**owner** and a **lifetime**; the daemon validates fingerprints and
cleans up on disconnect where the interface can detect it. Register
commands carry a reply channel - the interfaces return *typed results*,
not fire-and-forget.

## Registration payload

| Kind | Payload | Notes |
|------|---------|-------|
| `Pattern` | `ProfileFile` (usually built via `from_points`) | becomes a `TangiblePattern` with a fresh monotonic `PatternId` |
| `FieldSet` | `FieldSet` TOML (same as `--fields`) | rejected on overlapping fields or duplicate name |

```jsonc
// GraphQL registerPattern(input: {...}):
{ "name": "pawn",
  "points": [0,0, 50,0, 0,50],      // flat mm pin coords
  "toleranceMm": 5.0,
  "spawnOnPartial": false,          // optional - SPAWN_AS_PARTIAL.md
  "spawnStability": 15,             // optional - frames to confirm
  "stage": "game" }                 // optional - pipeline scope, default: general
```

`stage` scopes the pattern to a named pipeline stage: its matches
suppress only through that stage's policy (EVENT_PIPELINE.md, see
[`../proxy/PIPELINE.md`](../proxy/PIPELINE.md)). `null`/absent means the
general stage.

## Result types

`registerPattern` and `registerFieldSet` return GraphQL **unions** so a
rejection carries machine-readable detail instead of an opaque error:

| Union | Members |
|-------|---------|
| `RegisterPatternResult` | `Registered` → `PatternRegistered { patternId }` · `Rejected` → `RegistrationRejected` |
| `RegisterFieldSetResult` | `Registered` → `FieldSetRegistered { name }` · `Rejected` → `RegistrationRejected` |

`RegistrationRejected { reason, kind, overlap }` - `kind` is the
`RejectionKind` enum to switch on:

| `RejectionKind` | Meaning | `overlap` payload |
|-----------------|---------|-------------------|
| `PATTERN_NOT_FOUND` | Referenced ID is not registered | - |
| `FINGERPRINT_COLLISION` | Distance fingerprint collides with a loaded pattern within its tolerance | - |
| `FIELDS_OVERLAP` | Two fields of the set intersect | `FieldOverlap { first, second, region }` - the offending pair plus intersection region |
| `DUPLICATE_FIELD_SET` | A field set with this name is registered already | - |

The same rejections surface on D-Bus as `fdo::Error` carrying the
`RegistrationError` message.

## Rules

- **Fingerprint collision → reject**: a new pattern whose distance
  fingerprint matches an existing pattern within its tolerance is
  refused; nothing changes. Verify via `profiles`.
- **Monotonic IDs**: `PatternId`s count up; a freed ID is never reused -
  a stale `pattern_id` can never silently point at a different figure.
- **`UpdateProfile` keeps the ID**: replaces geometry/name/stage in
  place and applies to *new* matches; tracked tangibles keep their
  recorded pose.
- **`owner`**: free-form string; on D-Bus an empty owner defaults to the
  caller's unique bus name (read from the message header).
- **`lifetime`**: `session` (default) or `persistent`.

## Lifetimes & cleanup

| Lifetime | Behavior |
|----------|----------|
| `session` | lives until `unregister*`/`unregister_owner` or the owning D-Bus client's name vanishes |
| `persistent` | additionally written to `~/.config/tangible-objects/profiles.d/` (patterns) resp. `fields.d/` (field sets); merged into the startup load next run |

- **D-Bus**: the daemon binds every registration to the caller's unique
  bus name and watches `NameOwnerChanged` - a client exit removes all of
  its session registrations automatically.
- **GraphQL/MCP**: no connection lifecycle - call `unregister_owner` on
  app exit.

## Interface surface

| Operation | GraphQL | MCP | D-Bus |
|-----------|---------|-----|-------|
| register pattern | `registerPattern(input, owner, lifetime)` → `RegisterPatternResult` | `register_pattern` | `register_pattern(name, points, tol, owner, persistent, spawn_on_partial, spawn_stability, stage)` |
| register field set | `registerFieldSet(input, owner, lifetime)` → `RegisterFieldSetResult` | `register_field_set` | `register_field_set(toml, owner, persistent)` |
| update profile | `updateProfile(id, input)` | `update_profile` | `update_profile(id, name, points, tol, spawn_on_partial, spawn_stability, stage)` |
| unregister pattern | `unregisterPattern(id)` | `unregister_pattern` | `unregister_pattern(id)` |
| unregister field set | `unregisterFieldSet(name)` | `unregister_field_set` | `unregister_field_set(name)` |
| unregister owner | `unregisterOwner(owner)` | `unregister_owner` | `unregister_owner(owner)` |
| register stage | `registerStage(name)` | `register_stage` | `register_stage(name)` |
| unregister stage | `unregisterStage(name)` | `unregister_stage` | `unregister_stage(name)` |
| list profiles | `profiles(owner?, staticOnly?)` | `list_profiles` | `list_profiles` |
| list field sets | `fieldSets` | (snapshot/`field_sets`) | (snapshot/`field_sets`) |

`points` is always the flat `[x0,y0,x1,y1,...]` mm list; `tolerance` is a
scalar mm value (per-index tolerances come from file profiles, not the
quick-register payload). `register_stage`/`unregister_stage` manage
explicit stage scopes - a stage also springs into existence
implicitly when the first `stage = "…"` pattern registers and dies with
its last pattern unless it was registered explicitly.

## Where things live

- `Engine.patterns` - the live pattern pool (`add_pattern`/
  `update_pattern`/`remove_pattern`/`remove_by_owner`)
- `Engine.pattern_owners`/`field_set_owners` - owner bookkeeping
- `EngineHandle.profiles` - `RwLock`-shared `ProfileInfo` list so
  interface reads see registrations immediately
- `~/.config/tangible-objects/profiles.d` / `fields.d` - the persistent
  runtime store
- Lifecycle events (`PROFILE_REGISTERED`, `FIELD_SET_REGISTERED`,
  `STAGE_ADDED`, …) announce every applied change on the event stream -
  see [`../event/DOMAIN_EVENTS.md`](../event/DOMAIN_EVENTS.md)
