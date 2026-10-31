#[cfg(feature = "cynic")]
use crate::schema;
#[cfg(not(feature = "cynic"))]
use serde::Deserialize;
#[cfg(not(feature = "cynic"))]
use serde::Serialize;

/// Machine-readable category of a [`crate::RegistrationRejected`] result -
/// mirrors the `RegistrationError` variants so clients can switch on the
/// kind without parsing the `reason` string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "cynic"), derive(Serialize, Deserialize))]
#[cfg_attr(not(feature = "cynic"), serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "cynic", derive(cynic::Enum))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        rename_all = "SCREAMING_SNAKE_CASE"
    )
)]
// Under `cynic` the derive owns serde - same wire values either way.
pub enum RejectionKind {
    /// The referenced pattern ID is not registered.
    PatternNotFound,
    /// The profile fingerprint collides with a registered profile.
    FingerprintCollision,
    /// The field set contains overlapping fields - `overlap` carries
    /// the structured detail.
    FieldsOverlap,
    /// A field set with this name is already registered.
    DuplicateFieldSet,
}
