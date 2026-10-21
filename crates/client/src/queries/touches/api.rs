//! `touch_points()` method of [`GraphQLClient`].

use tangible_objects_model::TouchPoint;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::touches::query::TouchPointsQuery;

impl GraphQLClient {
    /// Active touch points (`touchPoints` query).
    pub async fn touch_points(&self) -> Result<Vec<TouchPoint>, ClientError> {
        self.call_query::<TouchPointsQuery, _, _>("touchPoints", ())
            .await
    }
}
