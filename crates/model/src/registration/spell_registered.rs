//! `SpellRegistered` - the `registerSpell` applied payload, echoing the
//! registered spell name so union clients can bind the acknowledgement
//! without tracking variables.

use serde::Deserialize;
use serde::Serialize;

use crate::SpellName;
#[cfg(feature = "cynic")]
use crate::schema;

/// `registerSpell` applied - the spell was registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "SpellRegistered",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct SpellRegistered {
    /// Name of the registered spell.
    pub name: SpellName,
}
