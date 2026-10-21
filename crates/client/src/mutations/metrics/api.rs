//! `reset_metrics()` method of [`GraphQLClient`].

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::metrics::mutation::ResetMetrics;

impl GraphQLClient {
    /// Resets the session metrics (`resetMetrics`).
    pub async fn reset_metrics(&self) -> Result<bool, ClientError> {
        self.call_mutation::<ResetMetrics, _, _>("resetMetrics", ())
            .await
    }
}
