//! `touchPoints` query - active touch contacts.

use serde_json::json;
use tangible_objects_model::TouchPoint;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Active touch points (`touchPoints` query).
    pub async fn touch_points(&self) -> Result<Vec<TouchPoint>, ClientError> {
        self.call(
            "touchPoints",
            "{ touchPoints { slot trackingId pos { x y } } }",
            json!({}),
        )
        .await
    }
}
