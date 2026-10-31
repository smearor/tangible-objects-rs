use crate::FieldName;
use crate::Region;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// Structured overlap detail of a `FIELDS_OVERLAP` registration
/// rejection: the offending field pair plus their intersection region -
/// the client can highlight the collision without parsing text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "FieldOverlap",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct FieldOverlap {
    /// First overlapping field.
    pub first: FieldName,
    /// Second overlapping field.
    pub second: FieldName,
    /// Intersection region of the two fields (millimeters).
    pub region: Region,
}
