# tangible-objects-client

Typed remote client for the `tangible-objects` daemon's GraphQL
endpoint. Covers the full schema surface - every query, mutation and
subscription behind a method, model types in, model types out.

- **Transport**: `reqwest` (HTTP POST) for queries/mutations,
  `graphql-transport-ws` over `tokio-tungstenite` for subscriptions
- **Types**: `tangible-objects-model` throughout - re-exported as
  `tangible_objects_client::model` so a second dependency is usually
  unnecessary

## Quickstart

```rust
use tangible_objects_client::{Endpoint, GraphQLClient};

#[tokio::main]
async fn main() -> Result<(), tangible_objects_client::ClientError> {
    // flag/env > client.toml > http://127.0.0.1:8080/graphql
    let client = GraphQLClient::new(Endpoint::resolve(None))?;

    for t in client.tangibles().await? {
        println!("{} @ ({:.1},{:.1})", t.pattern_name.as_str(), t.center.x, t.center.y);
    }
    Ok(())
}
```

## Endpoint resolution

`Endpoint::resolve(explicit)` wins in this order:

1. `Some(url)` (your `--endpoint` flag or `TANGIBLE_ENDPOINT`)
2. `endpoint` key in `~/.config/tangible-objects/client.toml`
   (`XDG_CONFIG_HOME` aware)
3. `Endpoint::DEFAULT` (`http://127.0.0.1:8080/graphql`)

The WebSocket URL for subscriptions is derived automatically
(`http(s)://` → `ws(s)://`) - pass the same endpoint everywhere.

## API map

| Section | Methods |
|---|---|
| Queries | `tangibles` `touch_points` `sets` `profiles` `device_info` `recording_status` `completion` `metrics` `message` `analyze_recording` `analyze_similarity` `recording_artifact` |
| Mutations | `set_filter_mode` `register_stage` `unregister_stage` `train_profile` `start_recording` `stop_recording` `set_record_label` `set_completion` `reset_metrics` `show_message` `set_gui_visible` `register_pattern` `update_profile` `unregister_pattern` `unregister_owner` `register_field_set` `unregister_field_set` |
| Subscriptions | `event_stream(&EventStreamQuery)` `tangible_events` `field_events` |

## Queries

```rust
let tangibles = client.tangibles().await?;
let touches = client.touch_points().await?;
let sets = client.sets().await?;
let device = client.device_info().await?;           // Option<DeviceInfo>

// Narrowed profile listing:
let query = ProfileQuery { owner: Some("my-app".into()), ..Default::default() };
let profiles = client.profiles(&query).await?;
```

## Mutations

```rust
use tangible_objects_client::model::{FilterMode, Lifetime};

// Suppression policy - `None` broadcasts to all stages:
client.set_filter_mode(FilterMode::Guarded, None, None).await?;

// Runtime profile registration:
let profile = model::ProfileFile::load(path)?;
client.register_pattern(&profile, None, Lifetime::Session).await?;

// Recording session:
client.start_recording("/tmp/rec.jsonl", None).await?;
client.set_record_label(Some("baseline")).await?;
client.stop_recording().await?;
```

## Subscriptions

Every subscription returns `Stream<Item = Result<TangibleEvent, ClientError>>`.

```rust
use futures_lite::StreamExt;
use tangible_objects_client::{EventStreamQuery, model::StreamEvent};

// Typed event stream (kinds must be non-empty - the server rejects
// an empty filter):
let query = EventStreamQuery {
    kinds: vec![StreamEvent::TangibleAdded, StreamEvent::TangibleMoved],
    max_hz: Some(30),
    ..Default::default()
};
let mut stream = client.event_stream(&query).await?;
while let Some(event) = stream.next().await {
    match event {
        Ok(e) if e.kind == model::EventKind::Lagged => {
            eprintln!("stream lagged - {} events dropped", e.skipped.unwrap_or_default())
        }
        Ok(e) => println!("{e:?}"),
        Err(err) => eprintln!("stream error: {err}"),
    }
}

// Convenience streams:
let mut tangibles = client.tangible_events(None).await?;
let mut fields = client.field_events(Some("workspace")).await?;
```

## Errors

`ClientError` (thiserror):

| Variant | Meaning |
|---|---|
| `Http` / `Ws` | transport failure (reqwest/tungstenite) |
| `Graphql(Value)` | server returned a GraphQL `errors` array |
| `Decode` | response didn't match the model type |
| `Protocol` | handshake or frame-protocol violation |

`LAGGED` events surface inside `TangibleEvent::kind`
(`EventKind::Lagged` + `skipped` count), not as errors - check
`e.kind` when consuming streams.

## Cargo

```toml
[dependencies]
tangible-objects-client = { path = "../crates/client" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
futures-lite = "2"
```

The CLI (`crates/cli`) and TUI (`crates/tui`) are complete usage
examples of this client.
