//! `EventStreamSub` - the cynic fragment for the `eventStream`
//! subscription. The derive builds the document and checks every
//! field and argument against `schema.graphql` at compile time.

use cynic::QueryFragment;
use tangible_objects_model::TangibleEvent;

use crate::subscriptions::events::query::EventStreamQueryFields;

/// Root-field wrapper for the `eventStream` subscription.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "SubscriptionRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "EventStreamQuery"
)]
pub(crate) struct EventStreamSub {
    /// `eventStream` field.
    #[cynic(rename = "eventStream")]
    #[arguments(kinds: $kinds, patternId: $pattern_id, setId: $set_id, maxHz: $max_hz)]
    event_stream: TangibleEvent,
}

#[cfg(test)]
mod tests {
    use super::EventStreamSub;
    use crate::EventStreamQuery;
    use cynic::StreamingOperation;

    /// The cynic-derived document declares the four schema arguments
    /// and selects the event payload fields.
    #[test]
    fn event_stream_document() {
        let query = StreamingOperation::<EventStreamSub, _>::subscription(EventStreamQuery::all())
            .query()
            .to_string();
        for field in [
            "$kinds: [StreamEvent!]!",
            "$patternId: PatternId",
            "$setId: TangibleSetId",
            "$maxHz: Int",
            "eventStream",
            "kind",
            "tangible",
            "setId",
            "skipped",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
