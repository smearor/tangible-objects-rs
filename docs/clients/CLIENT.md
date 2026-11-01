# `tangible-objects-client` - Typed GraphQL Client

The client library (`crates/client`, GRAPHQL_CLIENT.md) that
`tangible-objects-cli` and `tangible-objects-tui` build on. It covers
the full schema surface - all queries, mutations and subscriptions -
typed end to end: model types in, model types out.

## Key properties

- **Compile-time checked documents**: `cynic` query fragments are
  verified against `crates/graphql/schema.graphql` at build time - no
  hand-written query strings, schema drift fails compilation.
- **Subscriptions** run the `graphql-transport-ws` protocol over a raw
  `tokio-tungstenite` socket (`ws/` module).
- **No serde duplication**: response types deserialize directly into
  `tangible-objects-model` types (`TrackedTangible`, `FieldSet`, …).

```rust
let endpoint = Endpoint::resolve(None); // flag > env > client.toml > default
let client = GraphQLClient::new(endpoint)?;
for tangible in client.tangibles().await? {
    println!("{} @ ({:.1},{:.1})", tangible.pattern_name, tangible.center.x, tangible.center.y);
}
```

## Endpoint resolution

`Endpoint::resolve(flag_value)` precedence:

1. `--endpoint` argument (callers pass their CLI flag through)
2. `TANGIBLE_ENDPOINT` environment variable
3. `client.toml` config file
4. built-in default (`http://127.0.0.1:8080/graphql`)

## Layout

| Module | Content |
|--------|---------|
| `queries/` | `tangibles`, `touches`, `sets`, `profiles`, `device`, `recording`, `completion`, `metrics`, `message`, `field_sets`, `analysis` (recording/similarity/artifact) |
| `mutations/` | `filter`, `stage`, `pattern` (register/update/unregister/unregister_owner), `field_set` (register/unregister), `recording` (start/stop/set_label), `training`, `completion`, `metrics`, `message`, `gui` |
| `subscriptions/` | `events` (`eventStream`), `tangibles`, `fields` |
| `ws/` | `graphql-transport-ws` connection, operation and message framing |
| `endpoint.rs` / `config.rs` | endpoint resolution + `client.toml` |
| `error.rs` | `ClientError` |

Registration mutations return the typed unions
(`RegisterPatternResult`/`RegisterFieldSetResult`) so callers switch on
`RejectionKind` instead of parsing error strings
([`../profile/REGISTRATION.md`](../profile/REGISTRATION.md)).
