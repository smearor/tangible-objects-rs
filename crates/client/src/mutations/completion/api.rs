//! `set_completion()` method of [`GraphQLClient`].

use tangible_objects_model::CompletionConfig;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::completion::mutation::SetCompletion;
use crate::mutations::completion::variables::SetCompletionVariables;

impl GraphQLClient {
    /// Applies a completion configuration (`setCompletion`).
    pub async fn set_completion(&self, config: &CompletionConfig) -> Result<bool, ClientError> {
        self.call_mutation::<SetCompletion, _, _>(
            "setCompletion",
            SetCompletionVariables {
                config: config.into(),
            },
        )
        .await
    }
}
