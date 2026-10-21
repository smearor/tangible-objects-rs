//! `show_message()` method of [`GraphQLClient`].

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::message::mutation::ShowMessage;
use crate::mutations::message::variables::ShowMessageVariables;

impl GraphQLClient {
    /// Pushes a user-facing message (`showMessage`).
    pub async fn show_message(&self, text: &str) -> Result<bool, ClientError> {
        self.call_mutation::<ShowMessage, _, _>(
            "showMessage",
            ShowMessageVariables {
                text: text.to_string(),
            },
        )
        .await
    }
}
