//! `message` query - the user-facing GUI overlay text.

use serde_json::json;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// User-facing message set via `showMessage` (`message` query).
    pub async fn message(&self) -> Result<Option<String>, ClientError> {
        self.call("message", "{ message }", json!({})).await
    }
}
