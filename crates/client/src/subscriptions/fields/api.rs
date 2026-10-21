//! `field_events()` method of [`GraphQLClient`].

use cynic::StreamingOperation;
use futures_lite::Stream;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::GraphQLClient;
use crate::subscriptions::fields::sub::FieldEventsSub;
use crate::subscriptions::fields::variables::FieldEventsVariables;
use crate::ws::subscribe;

impl GraphQLClient {
    /// Occupancy field stream (`fieldEvents`, optionally narrowed to
    /// one field set).
    pub async fn field_events(
        &self,
        set: Option<&str>,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        let operation =
            StreamingOperation::<FieldEventsSub, _>::subscription(FieldEventsVariables {
                set: set.map(str::to_string),
            });
        subscribe(&self.endpoint, "fieldEvents", operation).await
    }
}
