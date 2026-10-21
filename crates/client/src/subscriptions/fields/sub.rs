//! `FieldEventsSub` - the cynic fragment for the `fieldEvents`
//! subscription.

use cynic::QueryFragment;
use tangible_objects_model::TangibleEvent;

use crate::subscriptions::fields::variables::FieldEventsVariablesFields;

/// Root-field wrapper for the `fieldEvents` subscription - cynic
/// derives the document and checks every field and argument against
/// `schema.graphql` at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "SubscriptionRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "FieldEventsVariables"
)]
pub(crate) struct FieldEventsSub {
    /// `fieldEvents` field.
    #[cynic(rename = "fieldEvents")]
    #[arguments(set: $set)]
    field_events: TangibleEvent,
}
