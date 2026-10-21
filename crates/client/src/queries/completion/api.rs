//! `completion()` method of [`GraphQLClient`].

use tangible_objects_model::CompletionConfig;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::completion::query::CompletionQuery;

impl GraphQLClient {
    /// Completion configuration (`completion` query).
    pub async fn completion(&self) -> Result<Option<CompletionConfig>, ClientError> {
        self.call_query::<CompletionQuery, _, _>("completion", ())
            .await
    }
}
