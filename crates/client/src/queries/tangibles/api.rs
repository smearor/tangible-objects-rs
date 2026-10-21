//! `tangibles()` method of [`GraphQLClient`].

use tangible_objects_model::TrackedTangible;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::tangibles::query::TangiblesQuery;

impl GraphQLClient {
    /// Currently tracked tangibles (`tangibles` query).
    pub async fn tangibles(&self) -> Result<Vec<TrackedTangible>, ClientError> {
        self.call_query::<TangiblesQuery, _, _>("tangibles", ())
            .await
    }
}
