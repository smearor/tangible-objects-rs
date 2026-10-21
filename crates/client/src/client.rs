//! `GraphQLClient` - the type plus its transport core. The method
//! surface lives in the crate-root topic modules `crate::queries`,
//! `crate::mutations` and `crate::subscriptions` (one file per theme).

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::json;

use crate::ClientError;
use crate::Endpoint;

/// Typed client for the daemon's GraphQL endpoint - every schema field
/// behind a method; model types in, model types out.
pub struct GraphQLClient {
    /// HTTP transport for queries and mutations.
    http: reqwest::Client,
    /// `http(s)://host:port/graphql` endpoint URL - subscriptions
    /// derive the `ws(s)://` transport from it.
    pub(crate) endpoint: Endpoint,
}

impl GraphQLClient {
    /// Builds a client for `endpoint` (`http://host:port/graphql` -
    /// `ws(s)://` is derived for subscriptions).
    pub fn new(endpoint: impl Into<Endpoint>) -> Result<Self, ClientError> {
        Ok(Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.into(),
        })
    }

    /// POSTs a document and returns `data[field]` decoded - the
    /// transport used by every query/mutation method in
    /// `crate::queries` and `crate::mutations`.
    pub(crate) async fn call<T: DeserializeOwned>(
        &self,
        field: &str,
        document: &str,
        variables: Value,
    ) -> Result<T, ClientError> {
        let response = self
            .http
            .post(self.endpoint.as_str())
            .json(&json!({ "query": document, "variables": variables }))
            .send()
            .await?
            .error_for_status()?;
        let body: Value = response.json().await?;
        if let Some(errors) = body.get("errors") {
            return Err(ClientError::Graphql(errors.clone()));
        }
        let data = body.get("data").cloned().unwrap_or(Value::Null);
        serde_json::from_value(data.get(field).cloned().unwrap_or(Value::Null))
            .map_err(ClientError::Decode)
    }
}

/// Serializes a string-valued enum into the schema's SCREAMING_SNAKE
/// enum name (`"guarded"`/`"Guarded"`/`"kebab-case"` → `"GUARDED"`).
pub(crate) fn gql_enum(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .map(|s| s.replace('-', "_").to_uppercase())
        .unwrap_or_default()
}

/// Recursively converts `snake_case` object keys to `camelCase` -
/// model serde shapes → GraphQL input shapes.
pub(crate) fn camelize(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| {
                    (
                        k.split('_')
                            .enumerate()
                            .map(|(i, part)| {
                                if i == 0 {
                                    part.to_string()
                                } else {
                                    let mut chars = part.chars();
                                    match chars.next() {
                                        Some(first) => {
                                            format!("{}{}", first.to_uppercase(), chars.as_str())
                                        }
                                        None => String::new(),
                                    }
                                }
                            })
                            .collect::<String>(),
                        camelize(v),
                    )
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(camelize).collect()),
        other => other,
    }
}
