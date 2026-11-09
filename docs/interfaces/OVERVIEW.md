# Interfaces: GraphQL · MCP · D-Bus

Three interfaces expose the running daemon in parallel, each enabled by
its own flag and backed by the same `EngineHandle` (broadcast
`DomainEvent`s, `watch<EngineSnapshot>`, `mpsc<ControlCommand>` - the
`runtime` crate). The GraphQL schema is the canonical surface:
`crates/graphql/schema.graphql` is checked in and the client crate's
`cynic` documents are verified against it at compile time.

| Interface | Enable | Transport | Endpoint / name |
|-----------|--------|-----------|-----------------|
| GraphQL | `--graphql-listen <addr>` | HTTP + WebSocket | `GET /` GraphiQL IDE · `POST /graphql` queries+mutations · `GET /graphql` `graphql-transport-ws` subscriptions |
| MCP | `--mcp-listen <addr>` | streamable HTTP (`/mcp`) + stdio | tools + `tangible://` resources |
| D-Bus | `--dbus` | session bus | service `org.tangibleobjects.Engine1` |

GraphQL and MCP are two independent axum servers, each on its own
thread and tokio runtime - enabling both on the same address fails with
*address already in use*. **No authentication** - LAN bindings only;
`recording_artifact` is restricted to `*.jsonl`/`*.evtest`/
`*.dataset.jsonl` paths without `..`.

## Read model: EngineSnapshot

Queries read a `watch`-published snapshot, never the device loop:

| Snapshot field | Content |
|----------------|---------|
| `tracked` | `TrackedTangible[]` - EMA-smoothed poses incl. `pending` |
| `touches` | `TouchPoint[]` of the last frame |
| `active_sets` | `TangibleSetId[]` |
| `device` | `DeviceInfo` (name + calibration) - set once per session |
| `recording` | `RecordingStatus` - active flag, label, frame/drop counters |
| `completion` | active `CompletionConfig` |
| `field_sets` | `FieldSet[]` - static (`--fields`) + registered |
| `dead_zones` | `DeadZone[]` - startup store set + registered |
| `metrics` | `SessionMetrics` ([`../analysis/METRICS.md`](../analysis/METRICS.md)) |
| `message` | current `showMessage` overlay text |
| `gui_visible` | GUI window state (`gui` feature) |
| `dead_zones_visible` | dead-zone overlay state (`gui` feature) |
| `background_alpha` | canvas opacity percent (`gui` feature) |
| `edge_margin_visible` | edge-margin overlay state (`gui` feature) |
| `fields_visible` | field-set overlay state (`gui` feature) |
| `ruler_visible` | ruler overlay state (`gui` feature) |
| `positions_visible` | positions overlay state (`gui` feature) |
| `touch_passthrough` | touch fall-through to the desktop except on GUI elements (`gui` feature, Wayland only) |
| `filter_mode` | selected suppression policy (`SetFilterMode`; honored by the proxy in `--filter`, display state elsewhere) |

## Operation surface

| Domain | GraphQL | MCP tool | D-Bus |
|--------|---------|----------|-------|
| Tangibles | `tangibles`, `touchPoints` | `list_tangibles`, `touch_points`, `get_pose(id)` | `list_tangibles`, `touch_points`, `get_pose(id)` |
| Sets | `sets` | `tangible://sets` | `sets` |
| Profiles | `profiles(owner?, staticOnly?)` | `list_profiles`, `tangible://profiles` | `list_profiles` |
| Field sets | `fieldSets` | `tangible://field-sets` | (snapshot) |
| Device | `deviceInfo` | `device_info` | `device_info` |
| Recording | `recordingStatus`, `recordingArtifact(path)` | `recording_status`, `recording_artifact` | `recording_status`, `recording_artifact` |
| Filter | `setFilterMode(mode, stability?, stage?)` | `set_filter_mode` | `set_filter_mode` |
| Stages | `registerStage`, `unregisterStage` | `register_stage`, `unregister_stage` | `register_stage`, `unregister_stage` |
| Recording ctrl | `startRecording(basePath, label?)`, `stopRecording`, `setRecordLabel` | `start_recording`, `stop_recording`, `set_record_label` | same names |
| Training | `trainProfile(name)` | `train_profile` | `train_profile` |
| Completion | `completion`, `setCompletion(config)` | `get_completion`, `set_completion` | `completion`, `set_completion` |
| Metrics | `metrics`, `resetMetrics` | `get_metrics`, `reset_metrics` | `metrics`, `reset_metrics` |
| Message | `message`, `showMessage(text)` | `show_message`, `tangible://message` | `message`, `show_message` |
| GUI | `setGuiVisible`, `setDisplayRotation(deg)`, `setDeadZonesVisible`, `setBackgroundAlpha(pct)`, `setEdgeMarginVisible`, `setFieldsVisible`, `setRulerVisible`, `setPositionsVisible`, `setTouchPassthrough` | `set_gui_visible`, `set_display_rotation`, `set_dead_zones_visible`, `set_background_alpha`, `set_edge_margin_visible`, `set_fields_visible`, `set_ruler_visible`, `set_positions_visible`, `set_touch_passthrough` | `set_gui_visible`, `set_display_rotation`, `set_dead_zones_visible`, `set_background_alpha`, `set_edge_margin_visible`, `set_fields_visible`, `set_ruler_visible`, `set_positions_visible`, `set_touch_passthrough` |
| Registration | `registerPattern`, `updateProfile`, `unregisterPattern`, `registerFieldSet`, `unregisterFieldSet`, `registerDeadZone`, `unregisterDeadZone`, `unregisterOwner` | `register_pattern`, `update_profile`, `unregister_pattern`, `register_field_set`, `unregister_field_set`, `register_dead_zone`, `unregister_dead_zone`, `unregister_owner` | same names |
| Analysis | `analyzeRecording(path, mode?, profilesDir?)`, `analyzeSimilarity(profilesDir?)` | `analyze_recording`, `analyze_similarity` | `analyze_recording`, `analyze_similarity` |
| Events | `eventStream(kinds, patternId?, setId?, maxHz?)`, `tangibleEvents`, `fieldEvents` | `tangible://` snapshots (no stream) | per-kind signals + `lifecycle_event` |

Registration result unions (`RegisterPatternResult`,
`RegisterFieldSetResult`, `RejectionKind`) are documented in
[`../profile/REGISTRATION.md`](../profile/REGISTRATION.md); the event
model and subscription semantics in
[`../event/DOMAIN_EVENTS.md`](../event/DOMAIN_EVENTS.md).

## MCP resources

Read-only snapshots under `tangible://`: `tangible://tangibles`,
`touch-points`, `sets`, `profiles`, `field-sets`, `recording`,
`message` - the MCP counterpart of the snapshot queries.

## D-Bus notes

- Service `org.tangibleobjects.Engine1` on the **session** bus - no TCP
  port; permissions follow the bus policy.
- `register_*` calls read the caller's unique bus name from the message
  header when `owner` is empty; the daemon watches `NameOwnerChanged`
  and drops the caller's session registrations on exit
  ([`../profile/REGISTRATION.md`](../profile/REGISTRATION.md)).
- `lifecycle_event` carries the `LifecycleEvent` as a JSON string.
- Structured types use `zvariant::Type` signatures; `device_info`/
  `recording_status` return key-value lists.

## Clients

- `tangible-objects-client` - typed `cynic` GraphQL client library
  ([`../clients/CLIENT.md`](../clients/CLIENT.md))
- `tangible-objects-cli` - scriptable CLI on top of it
  ([`../clients/CLI.md`](../clients/CLI.md))
- `tangible-objects-tui` - terminal dashboard
  ([`../clients/TUI.md`](../clients/TUI.md))
