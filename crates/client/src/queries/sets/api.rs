//! `sets()` method of [`GraphQLClient`].

use tangible_objects_model::TangibleSetState;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::sets::query::SetsQuery;

impl GraphQLClient {
    /// Registered tangible-object sets with activation (`sets` query).
    pub async fn sets(&self) -> Result<Vec<TangibleSetState>, ClientError> {
        self.call_query::<SetsQuery, _, _>("sets", ()).await
    }
}
