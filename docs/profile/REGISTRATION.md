# Dynamic Profile Registration

Applications register figure profiles and occupancy field sets at
runtime instead of shipping files in `profiles/`. Registrations carry an
**owner** and a **lifetime**; the daemon validates fingerprints and
cleans up on disconnect where the interface can detect it.

## Registration payload

| Kind | Payload | Notes |
|------|---------|-------|
| `Pattern` | `ProfileFile` (usually built via `from_points`) | becomes a `TangiblePattern` with a fresh monotonic `PatternId` |
| `FieldSet` | `FieldSet` TOML (same as `--fields`) | rejected on overlapping fields or duplicate name |

```jsonc
// GraphQL registerPattern / D-Bus register_pattern:
{ "name": "pawn",
  "points": [0,0, 50,0, 0,50],   // flat mm pin coords
  "tolerance_mm": 5.0 }
```

## Rules

- **Fingerprint collision → reject**: a new pattern whose distance
  fingerprint matches an existing pattern within its tolerance is
  refused; nothing changes. Verify via `profiles`.
- **Monotonic IDs**: `PatternId`s count up; a freed ID is never reused -
  a stale `pattern_id` can never silently point at a different figure.
- **`UpdateProfile` keeps the ID**: replaces geometry/name in place and
  applies to *new* matches; tracked tangibles keep their recorded pose.
- **`owner`**: free-form string; on D-Bus an empty owner defaults to the
  caller's unique bus name.
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
| register pattern | `registerPattern(input, owner, lifetime)` | `register_pattern` | `register_pattern(name, points, tol, owner, persistent)` |
| register field set | `registerFieldSet(input, owner, lifetime)` | `register_field_set(toml)` | `register_field_set(toml, owner, persistent)` |
| update profile | `updateProfile(id, input)` | `update_profile` | `update_profile(id, name, points, tol)` |
| unregister pattern | `unregisterPattern(id)` | `unregister_pattern` | `unregister_pattern(id)` |
| unregister field set | `unregisterFieldSet(name)` | `unregister_field_set` | `unregister_field_set(name)` |
| unregister owner | `unregisterOwner(owner)` | `unregister_owner` | `unregister_owner(owner)` |
| list | `profiles(owner?, staticOnly?)` | `list_profiles` | `list_profiles` |

`points` is always the flat `[x0,y0,x1,y1,...]` mm list; `tolerance` is a
scalar mm value (per-index tolerances come from file profiles, not the
quick-register payload).

## Where things live

- `Engine.patterns` - the live pattern pool (`add_pattern`/
  `update_pattern`/`remove_pattern`/`remove_by_owner`)
- `Engine.pattern_owners`/`field_set_owners` - owner bookkeeping
- `EngineHandle.profiles` - `RwLock`-shared `ProfileInfo` list so
  interface reads see registrations immediately
- `~/.config/tangible-objects/profiles.d` / `fields.d` - the persistent
  runtime store
