//! `metrics` query - accumulated session metrics.

use serde_json::json;
use tangible_objects_model::SessionMetrics;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Session metrics (`metrics` query - `Json` passthrough, hence
    /// the serde-shaped `SessionMetrics`).
    pub async fn metrics(&self) -> Result<SessionMetrics, ClientError> {
        self.call("metrics", "{ metrics }", json!({})).await
    }
}
