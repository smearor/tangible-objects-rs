//! `TangibleEventsSub` - the cynic fragment for the
//! `tangibleEvents` subscription.

use cynic::QueryFragment;
use tangible_objects_model::TangibleEvent;

use crate::subscriptions::tangibles::variables::TangibleEventsVariablesFields;

/// Root-field wrapper for the `tangibleEvents` subscription - cynic
/// derives the document and checks every field and argument against
/// `schema.graphql` at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "SubscriptionRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "TangibleEventsVariables"
)]
pub(crate) struct TangibleEventsSub {
    /// `tangibleEvents` field.
    #[cynic(rename = "tangibleEvents")]
    #[arguments(patternId: $pattern_id)]
    tangible_events: TangibleEvent,
}

#[cfg(test)]
mod tests {
    use super::TangibleEventsSub;
    use crate::subscriptions::tangibles::variables::TangibleEventsVariables;
    use cynic::StreamingOperation;

    /// The cynic-derived document declares `$patternId` and selects
    /// the event payload fields.
    #[test]
    fn tangible_events_document() {
        let query =
            StreamingOperation::<TangibleEventsSub, _>::subscription(TangibleEventsVariables {
                pattern_id: None,
            })
            .query()
            .to_string();
        for field in [
            "$patternId: PatternId",
            "tangibleEvents",
            "kind",
            "tangible",
            "name",
            "patternId",
            "detail",
            "skipped",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
