//! `ProfileLookupVariables` - the variables of `profileLookup`.

use cynic::QueryVariables;

use crate::mutations::pattern::input::RegisterPatternInput;

/// Variables of `profileLookup` - the candidate profile geometry.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ProfileLookupVariables {
    /// The candidate profile - the same input `registerPattern` takes.
    pub input: RegisterPatternInput,
}
