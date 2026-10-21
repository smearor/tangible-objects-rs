//! `fieldEvents` subscription - occupancy-field events.

use futures_lite::Stream;
use serde_json::json;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::GraphQLClient;
use crate::ws::subscribe;

impl GraphQLClient {
    /// Occupancy field stream (`fieldEvents`, optionally narrowed to
    /// one field set).
    pub async fn field_events(
        &self,
        set: Option<&str>,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        subscribe(
            &self.endpoint,
            "fieldEvents",
            "subscription($set: String) { fieldEvents(set: $set) { kind fieldSet field count name detail skipped } }",
            json!({ "set": set }),
        )
        .await
    }
}
