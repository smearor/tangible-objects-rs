//! `resetMetrics` mutation - clears the session counters.

use serde_json::json;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Resets the session metrics (`resetMetrics`).
    pub async fn reset_metrics(&self) -> Result<bool, ClientError> {
        self.call("resetMetrics", "mutation { resetMetrics }", json!({}))
            .await
    }
}
