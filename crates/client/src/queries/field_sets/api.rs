//! `field_sets()` method of [`GraphQLClient`].

use tangible_objects_model::FieldSet;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::field_sets::query::FieldSetsQuery;

impl GraphQLClient {
    /// Registered occupancy field sets with their cells (`fieldSets`
    /// query).
    pub async fn field_sets(&self) -> Result<Vec<FieldSet>, ClientError> {
        self.call_query::<FieldSetsQuery, _, _>("fieldSets", ())
            .await
    }
}
