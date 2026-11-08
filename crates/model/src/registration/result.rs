use crate::DeadZoneName;
use crate::FieldSetName;
use crate::RegistrationRejected;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// `registerFieldSet` applied - echoes the registered set name so union
/// clients can bind the acknowledgement without tracking variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "FieldSetRegistered",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct FieldSetRegistered {
    /// Name of the registered field set.
    pub name: FieldSetName,
}

/// `registerFieldSet` result union - the applied outcome either
/// registered the set or carries the typed rejection (structured
/// overlap detail for `FIELDS_OVERLAP`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RegisterFieldSetResult {
    /// The field set was registered.
    Registered(FieldSetRegistered),
    /// The registration was rejected - `rejection` says why, with
    /// structured detail.
    Rejected(RegistrationRejected),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "DeadZoneRegistered",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct DeadZoneRegistered {
    /// Name of the registered dead zone.
    pub name: DeadZoneName,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RegisterDeadZoneResult {
    /// The dead zone was registered.
    Registered(DeadZoneRegistered),
    /// The registration was rejected - `rejection` says why, with
    /// structured detail.
    Rejected(RegistrationRejected),
}
