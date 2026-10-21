//! `EventOperation` - the [`GraphqlOperation`] for the serde decode
//! path: serializes the `StreamingOperation` payload verbatim and
//! decodes `payload.data[field]` through the model's `Deserialize`.

use graphql_ws_client::graphql::GraphqlOperation;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use tangible_objects_model::TangibleEvent;

/// Serializes to the subscribe payload (`{query, variables,
/// operationName}` - the `cynic::StreamingOperation` shape) and
/// decodes each `next` frame's `data[field]` through serde.
pub(crate) struct EventOperation {
    /// Serialized operation payload.
    payload: Value,
    /// Root field inside `data` carrying the `TangibleEvent`.
    field: &'static str,
}

impl EventOperation {
    /// Wraps a serialized operation for field `field`.
    pub(crate) fn new(payload: Value, field: &'static str) -> Self {
        Self { payload, field }
    }
}

impl Serialize for EventOperation {
    /// Emits the inner operation payload verbatim.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.payload.serialize(serializer)
    }
}

impl GraphqlOperation for EventOperation {
    type Response = TangibleEvent;
    type Error = serde_json::Error;

    /// `data` is the frame's `payload` - `{data: {field: event}}` -
    /// the field extracts via the model's `Deserialize`.
    fn decode(&self, data: Value) -> Result<Self::Response, Self::Error> {
        serde_json::from_value(
            data.get("data")
                .and_then(|data| data.get(self.field))
                .cloned()
                .unwrap_or(Value::Null),
        )
    }
}
