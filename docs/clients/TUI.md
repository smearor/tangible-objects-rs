# `tangible-objects-tui` - Terminal Dashboard

Ratatui/crossterm dashboard for a running daemon (`crates/tui`) - a live
read view over the GraphQL endpoint, built on
[`tangible-objects-client`](CLIENT.md) (CLI_CLIENT.md; the broader TUI
design is `concepts/planned/TUI.md`).

```bash
tangible-objects-tui [--endpoint http://host:8080/graphql]
```

Same endpoint resolution as the CLI: `--endpoint` >
`TANGIBLE_ENDPOINT` > `client.toml` > localhost default. The daemon
must run with `--graphql-listen`.

## Layout

Two tabs, switched with `1`/`2`:

| Tab | Content |
|-----|---------|
| `Overview` | occupancy columns: slots, tracked tangibles, fields and field sets |
| `Live` | Braille-rendered surface canvas + profiles/metrics/event-log panes |

The event log tail-subscribes the domain event stream
([`../event/DOMAIN_EVENTS.md`](../event/DOMAIN_EVENTS.md)); snapshot
queries refresh the panes. `q`/Esc leaves the alternate screen and
restores the terminal.

## i18n

UI strings come from Fluent (`i18n/{de,en}/tangible-objects-tui.ftl` via
`i18n-embed` + `i18n-embed-fl`), same mechanism as the embedded GUI -
see [`../i18n.md`](../i18n.md).
