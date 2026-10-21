//! `eventStream` subscription - the filtered domain-event stream.

use futures_lite::Stream;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::EventStreamQuery;
use crate::GraphQLClient;
use crate::ws::subscribe;

impl GraphQLClient {
    /// Filtered domain-event stream (`eventStream` subscription).
    pub async fn event_stream(
        &self,
        query: &EventStreamQuery,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        subscribe(
            &self.endpoint,
            "eventStream",
            "subscription($kinds: [StreamEvent!]!, $patternId: PatternId, $setId: TangibleSetId, $maxHz: Int) { eventStream(kinds: $kinds, patternId: $patternId, setId: $setId, maxHz: $maxHz) { kind tangible { id patternId patternName center { x y } rotationRad pending } setId fieldSet field count frames patternId name owner detail skipped } }",
            query.to_variables(),
        )
        .await
    }
}
