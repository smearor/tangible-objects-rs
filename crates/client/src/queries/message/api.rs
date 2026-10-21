//! `message()` method of [`GraphQLClient`].

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::message::query::MessageQuery;

impl GraphQLClient {
    /// User-facing message set via `showMessage` (`message` query).
    pub async fn message(&self) -> Result<Option<String>, ClientError> {
        self.call_query::<MessageQuery, _, _>("message", ()).await
    }
}
