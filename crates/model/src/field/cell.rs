#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

use crate::FieldName;
use crate::Region;

/// A single occupancy cell of a [`crate::FieldSet`]: a named region whose
/// touch contacts count as occupation.
///
/// Serialized flattened - a TOML `[[fields]]` entry carries `name` plus
/// the region bounds directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "Field",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct Field {
    /// Cell name (e.g. `"a1"` on a chess board).
    pub name: FieldName,
    /// Cell bounds in the set's coordinate space (screen mm, or
    /// window-relative / normalized when the set has an anchor).
    #[serde(flatten)]
    pub region: Region,
}
