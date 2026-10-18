# Field Sets: Occupancy Grids for Single-Contact Tangibles

A field set declares a named grid of occupancy cells for anonymous
1-contact markers (SINGLE_CONTACT_FIELD.md). Detection is *occupancy*:
`placed` when a cell gains a contact, `lifted` when it becomes empty -
debounced over `stability` consecutive frames in both directions.

## File format

```toml
name = "chess"
stability = 2            # optional, default 2 frames
suppress = false         # optional: contacts inside fields never reach
                         # the OS (filter-proxy suppression)

[window]                 # optional: fields become window-relative
x_offset = 50.0          # window's top-left corner, screen mm
y_offset = 30.0
width = 400.0            # optional: fields are normalized 0.0..=1.0
height = 400.0

[[fields]]
name = "a1"
x_min = 0.0
y_min = 0.0
x_max = 100.0            # mm (or 0.0..=1.0 when width/height are set)
y_max = 100.0
```

- Fields within a set must be **disjoint** - overlapping sets are rejected
  at load.
- Without `[window]` field coordinates are screen-absolute millimeters.
- With `[window]` they are relative to `x_offset`/`y_offset`; with
  `width`/`height` additionally normalized (`0.0..=1.0` of the window).
- `suppress = true` merges the field's contacts into the proxy's
  suppression set in `--filter` mode - a chess marker no longer taps the
  UI. Suppression is immediate (not debounced).

## Runtime registration

Field sets register like profiles (DYNAMIC_PROFILES.md): GraphQL
`registerFieldSet`, MCP `register_field_set` (TOML string), D-Bus
`register_field_set` - with `owner`, `lifetime` (`persistent` writes to
`~/.config/tangible-objects/fields.d/` and reloads on startup) and
`unregister_field_set`/`unregister_owner` removal.

## Usage

```bash
--live   --fields fields/chess.toml      # one set
--filter --fields fields/                # all .toml in a directory
```

## Events

| Event | Meaning |
|-------|---------|
| `FieldPlaced` | cell count went 0 → n after `stability` frames |
| `FieldLifted` | cell count went n → 0 after `stability` frames |
| `FieldCountChanged` | debounced count changed (multi-contact mode) |

Interfaces: GraphQL `EventKind` `FIELD_PLACED`/`FIELD_LIFTED`/
`FIELD_COUNT_CHANGED` (payload `fieldSet`, `field`, `count`), D-Bus
signals `field_placed`/`field_lifted`/`field_count_changed`.
