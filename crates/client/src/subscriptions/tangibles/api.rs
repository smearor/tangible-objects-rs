//! `tangible_events()` method of [`GraphQLClient`].

use cynic::StreamingOperation;
use futures_lite::Stream;
use tangible_objects_model::PatternId;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::GraphQLClient;
use crate::subscriptions::tangibles::sub::TangibleEventsSub;
use crate::subscriptions::tangibles::variables::TangibleEventsVariables;
use crate::ws::subscribe;

impl GraphQLClient {
    /// Tangible lifecycle + motion stream (`tangibleEvents`).
    pub async fn tangible_events(
        &self,
        pattern_id: Option<PatternId>,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        let operation =
            StreamingOperation::<TangibleEventsSub, _>::subscription(TangibleEventsVariables {
                pattern_id,
            });
        subscribe(&self.endpoint, "tangibleEvents", operation).await
    }
}
