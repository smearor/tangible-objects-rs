//! `sets` query - Tangible Object Set states.

use serde_json::json;
use tangible_objects_model::TangibleSetState;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Tangible Object Set states (`sets` query).
    pub async fn sets(&self) -> Result<Vec<TangibleSetState>, ClientError> {
        self.call(
            "sets",
            "{ sets { set { id name mandatory additional } active } }",
            json!({}),
        )
        .await
    }
}
