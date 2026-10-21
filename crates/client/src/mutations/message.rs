//! `showMessage` mutation - pushes the user-facing overlay text.

use serde_json::json;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Pushes a user-facing message (`showMessage`).
    pub async fn show_message(&self, text: &str) -> Result<bool, ClientError> {
        self.call(
            "showMessage",
            "mutation($text: String!) { showMessage(text: $text) }",
            json!({ "text": text }),
        )
        .await
    }
}
