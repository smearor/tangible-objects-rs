//! `ProfileLookupQuery` - the cynic fragment for the `profileLookup`
//! query.

use tangible_objects_model::ProfileLookup;

use crate::queries::profile_lookup::variables::ProfileLookupVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `profileLookup` - cynic derives the document
/// and checks every selected field and argument against
/// `schema.graphql` at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ProfileLookupVariables"
)]
pub(crate) struct ProfileLookupQuery {
    /// `profileLookup` field.
    #[cynic(rename = "profileLookup")]
    #[arguments(input: $input)]
    profile_lookup: ProfileLookup,
}
