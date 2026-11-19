//! `filter_mode()` method of [`GraphQLClient`].

use tangible_objects_model::FilterMode;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::filter::query::FilterModeQuery;

impl GraphQLClient {
    /// The filter proxy's current suppression policy (`filterMode`
    /// query).
    pub async fn filter_mode(&self) -> Result<FilterMode, ClientError> {
        self.call_query::<FilterModeQuery, _, _>("filterMode", ())
            .await
    }
}
