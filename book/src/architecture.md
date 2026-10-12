# Architecture

## Crate Dependency Graph

Edges point from a crate to the crate it depends on.

```mermaid
graph BT
    model["tangible-objects-model"]
    engine["tangible-objects-engine"]
    runtime["tangible-objects-runtime"]
    graphql["tangible-objects-graphql"]
    dbus["tangible-objects-dbus"]
    mcp["tangible-objects-mcp"]
    daemon["tangible-objects (daemon)"]

    engine --> model
    runtime --> engine
    runtime --> model
    graphql --> runtime
    graphql --> engine
    graphql --> model
    dbus --> runtime
    dbus --> engine
    dbus --> model
    mcp --> runtime
    mcp --> model
    daemon --> engine
    daemon --> model
    daemon --> runtime
    daemon --> graphql
    daemon --> dbus
    daemon --> mcp

    classDef base fill:#2d4263,stroke:#3f6db4,color:#fff
    classDef iface fill:#2d5a3d,stroke:#4a9e63,color:#fff
    classDef top fill:#5a3d2d,stroke:#b47a3f,color:#fff
    class model base
    class graphql,dbus,mcp iface
    class daemon top
```

| Crate | Role | Depends on |
| ----- | ---- | ---------- |
| `tangible-objects-model` | Domain types: `TrackedTangible`, `TangiblePattern`, `TangibleSet`, `TouchPoint`, `SetAction` | — |
| `tangible-objects-engine` | Consumes `RawEvent`s, detects tangibles, emits `EngineFrame`/`EngineEvent` | `model` |
| `tangible-objects-runtime` | Event hub: `broadcast<DomainEvent>`, `watch<EngineSnapshot>`, `mpsc<ControlCommand>`; publishes `EngineHandle` | `engine`, `model` |
| `tangible-objects-graphql` | GraphQL query/mutation/subscription via axum + async-graphql | `runtime`, `engine`, `model` |
| `tangible-objects-dbus` | `org.tangibleobjects.Engine1` on the session bus; methods + signals | `runtime`, `engine`, `model` |
| `tangible-objects-mcp` | MCP tools + `tangible://` resources (stdio, streamable HTTP) | `runtime`, `model` |
| `tangible-objects` (daemon) | Binary; owns the device loop, publishes frames, spawns the interface servers | all |

## Interface Endpoints

The daemon can expose three interfaces in parallel, each enabled by a CLI flag:

- **GraphQL** (`--graphql-listen <addr>`): axum server on its own thread and
  tokio runtime. Routes: `GET /` (GraphiQL IDE), `POST /graphql` (queries and
  mutations), `GET /graphql` (WebSocket subscriptions).
- **D-Bus** (`--dbus`): service `org.tangibleobjects.Engine1` on the session bus.
  No TCP port.
- **MCP** (`--mcp-listen <addr>`): axum server on its own thread and tokio
  runtime. Route: `/mcp` (streamable HTTP transport).

GraphQL and MCP are two fully independent HTTP servers: each binds its own
`TcpListener` on its own address and port. Enabling both with the same address
fails with *address already in use*.
