//! `ProfilesQuery` - the cynic fragment for the `profiles` query.

use tangible_objects_model::ProfileInfo;

use crate::queries::profiles::variables::ProfileQueryFields;
use cynic::QueryFragment;

/// Root-field wrapper for the `profiles` query - cynic derives the
/// document and checks every selected field and argument against
/// `schema.graphql` at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ProfileQuery"
)]
pub(crate) struct ProfilesQuery {
    /// `profiles` field.
    #[arguments(owner: $owner, staticOnly: $static_only)]
    profiles: Vec<ProfileInfo>,
}
