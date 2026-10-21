//! `metrics()` method of [`GraphQLClient`].

use tangible_objects_model::SessionMetrics;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::metrics::query::MetricsQuery;

impl GraphQLClient {
    /// Session metrics (`metrics` query).
    pub async fn metrics(&self) -> Result<SessionMetrics, ClientError> {
        self.call_query::<MetricsQuery, _, _>("metrics", ()).await
    }
}
