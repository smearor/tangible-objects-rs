//! `completion` query - the missing-pin completion configuration.

use serde_json::json;
use tangible_objects_model::CompletionConfig;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Completion configuration (`completion` query).
    pub async fn completion(&self) -> Result<Option<CompletionConfig>, ClientError> {
        self.call(
            "completion",
            "{ completion { mode radiusFactor maxMissing persist affinityFactor smoothingAlpha } }",
            json!({}),
        )
        .await
    }
}
