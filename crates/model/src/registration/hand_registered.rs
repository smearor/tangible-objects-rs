//! `HandProfileRegistered` - the `registerHandProfile` applied payload,
//! echoing the registered handprint name so union clients can bind
//! the acknowledgement without tracking variables.

use serde::Deserialize;
use serde::Serialize;

use crate::HandName;
#[cfg(feature = "cynic")]
use crate::schema;

/// `registerHandProfile` applied - the handprint was registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "HandProfileRegistered",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct HandProfileRegistered {
    /// Name of the registered handprint.
    pub name: HandName,
}
