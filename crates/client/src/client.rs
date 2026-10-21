//! `GraphQLClient` - the type plus its transport core. The method
//! surface lives in the crate-root topic modules `crate::queries`,
//! `crate::mutations` and `crate::subscriptions` (one file per theme).

use cynic::Operation;
use cynic::QueryFragment;
use cynic::QueryVariables;
use cynic::schema::MutationRoot;
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::ClientError;
use crate::Endpoint;

/// Typed client for the daemon's GraphQL endpoint - every schema field
/// behind a method; model types in, model types out.
pub struct GraphQLClient {
    /// HTTP transport for queries and mutations.
    pub(crate) http: Client,
    /// `http(s)://host:port/graphql` endpoint URL - subscriptions
    /// derive the `ws(s)://` transport from it.
    pub(crate) endpoint: Endpoint,
}

impl GraphQLClient {
    /// Builds a client for `endpoint` (`http://host:port/graphql` -
    /// `ws(s)://` is derived for subscriptions).
    pub fn new(endpoint: impl Into<Endpoint>) -> Result<Self, ClientError> {
        Ok(Self {
            http: Client::new(),
            endpoint: endpoint.into(),
        })
    }

    /// Builds a cynic query operation for the `F` fragment and POSTs
    /// it, returning `data[field]` decoded - the transport for
    /// `cynic::QueryFragment`-derived queries.
    pub(crate) async fn call_query<F, V, T>(
        &self,
        field: &str,
        variables: V,
    ) -> Result<T, ClientError>
    where
        F: QueryFragment,
        F::SchemaType: cynic::schema::QueryRoot,
        V: QueryVariables,
        Operation<F, V>: Serialize,
        T: DeserializeOwned,
    {
        self.post_and_decode(&Operation::<F, V>::query(variables), field)
            .await
    }

    /// Builds a cynic mutation operation for the `F` fragment and
    /// POSTs it, returning `data[field]` decoded - the transport for
    /// `cynic::QueryFragment`-derived mutations.
    pub(crate) async fn call_mutation<F, V, T>(
        &self,
        field: &str,
        variables: V,
    ) -> Result<T, ClientError>
    where
        F: QueryFragment,
        F::SchemaType: MutationRoot,
        V: QueryVariables,
        Operation<F, V>: Serialize,
        T: DeserializeOwned,
    {
        self.post_and_decode(&Operation::<F, V>::mutation(variables), field)
            .await
    }

    /// Shared transport tail: POST the payload, surface `errors` as
    /// [`ClientError::Graphql`], decode `data[field]` through serde.
    async fn post_and_decode<T: DeserializeOwned>(
        &self,
        payload: &impl Serialize,
        field: &str,
    ) -> Result<T, ClientError> {
        let response = self
            .http
            .post(self.endpoint.as_str())
            .json(payload)
            .send()
            .await?
            .error_for_status()?;
        let body: Value = response.json().await?;
        if let Some(errors) = body.get("errors") {
            return Err(ClientError::Graphql(errors.clone()));
        }
        let data = body.get("data").filter(|d| !d.is_null());
        let value = data.and_then(|d| d.get(field)).ok_or_else(|| match data {
            None => ClientError::NoData,
            Some(_) => ClientError::FieldNotFound(field.to_string()),
        })?;
        serde_json::from_value(value.clone()).map_err(ClientError::Decode)
    }
}
