//! `tangibles` query - currently tracked figures.

use serde_json::json;
use tangible_objects_model::TrackedTangible;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Currently tracked tangibles (`tangibles` query).
    pub async fn tangibles(&self) -> Result<Vec<TrackedTangible>, ClientError> {
        self.call(
            "tangibles",
            "{ tangibles { id patternId patternName center { x y } rotationRad pending } }",
            json!({}),
        )
        .await
    }
}
