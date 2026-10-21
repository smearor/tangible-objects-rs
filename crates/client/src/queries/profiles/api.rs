//! `profiles()` method of [`GraphQLClient`].

pub use crate::queries::profiles::variables::ProfileQuery;

use tangible_objects_model::ProfileInfo;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::profiles::query::ProfilesQuery;

impl GraphQLClient {
    /// Loaded/registered figure profiles (`profiles` query, narrowed
    /// by `query`).
    pub async fn profiles(&self, query: &ProfileQuery) -> Result<Vec<ProfileInfo>, ClientError> {
        self.call_query::<ProfilesQuery, _, _>("profiles", query.clone())
            .await
    }
}
