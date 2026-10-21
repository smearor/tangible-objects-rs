//! `event_stream()` method of [`GraphQLClient`].

use crate::ClientError;
use crate::GraphQLClient;
pub use crate::subscriptions::events::query::EventStreamQuery;
use crate::subscriptions::events::sub::EventStreamSub;
use crate::ws::subscribe;
use cynic::StreamingOperation;
use futures_lite::Stream;
use tangible_objects_model::TangibleEvent;

impl GraphQLClient {
    /// Filtered domain-event stream (`eventStream` subscription).
    pub async fn event_stream(
        &self,
        query: &EventStreamQuery,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        let operation = StreamingOperation::<EventStreamSub, _>::subscription(query.clone());
        subscribe(&self.endpoint, "eventStream", operation).await
    }
}
