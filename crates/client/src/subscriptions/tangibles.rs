//! `tangibleEvents` subscription - tangible lifecycle + motion
//! (deprecated convenience; `event_stream` is the typed path).

use futures_lite::Stream;
use serde_json::json;
use tangible_objects_model::PatternId;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::GraphQLClient;
use crate::ws::subscribe;

impl GraphQLClient {
    /// Tangible lifecycle + motion stream (`tangibleEvents`).
    pub async fn tangible_events(
        &self,
        pattern_id: Option<PatternId>,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        subscribe(
            &self.endpoint,
            "tangibleEvents",
            "subscription($patternId: PatternId) { tangibleEvents(patternId: $patternId) { kind tangible { id patternId patternName center { x y } rotationRad pending } name patternId detail skipped } }",
            json!({ "patternId": pattern_id.map(|p| p.as_u32()) }),
        )
        .await
    }
}
