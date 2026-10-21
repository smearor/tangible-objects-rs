use super::TangibleSet;
use super::TangibleSetId;
use crate::PatternId;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

/// A [`TangibleSet`] definition joined with its live activation state.
///
/// The `active` flag is runtime state derived by the [`SetTracker`] and not
/// part of the persisted set definition; this type bundles both for
/// interface-facing queries. Serializes flat as `{id, name, mandatory,
/// additional, active}`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "TangibleSetState",
        no_deserialize
    )
)]
pub struct TangibleSetState {
    /// The set definition; serialized inline via `#[serde(flatten)]`.
    #[serde(flatten)]
    pub set: TangibleSet,
    /// Whether the set is currently active on the surface.
    pub active: bool,
}

#[cfg(feature = "zbus")]
impl zvariant::Type for TangibleSetState {
    /// Flat `(id, name, mandatory, additional, active)` - matches the
    /// `#[serde(flatten)]` field order.
    const SIGNATURE: &'static zvariant::Signature = &zvariant::signature!("(usaauaub)");
}

impl<'de> Deserialize<'de> for TangibleSetState {
    /// Accepts both wire shapes: the GraphQL `{set: {...}, active}`
    /// nesting (what the schema emits) and the legacy flat form
    /// `{id, name, mandatory, additional, active}`.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            /// `{set: {id, name, mandatory, additional}, active}`.
            Nested {
                /// The nested set definition.
                set: TangibleSet,
                /// Activation flag.
                active: bool,
            },
            /// `{id, name, mandatory, additional, active}` - the flat
            /// serialized form.
            Flat {
                /// Set identifier.
                id: TangibleSetId,
                /// Set name.
                name: String,
                /// Mandatory member patterns.
                mandatory: Vec<PatternId>,
                /// Optional member patterns.
                additional: Vec<PatternId>,
                /// Activation flag.
                active: bool,
            },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Nested { set, active } => Self { set, active },
            Wire::Flat {
                id,
                name,
                mandatory,
                additional,
                active,
            } => Self {
                set: TangibleSet {
                    id,
                    name,
                    mandatory,
                    additional,
                },
                active,
            },
        })
    }
}

impl From<(&TangibleSet, bool)> for TangibleSetState {
    fn from((set, active): (&TangibleSet, bool)) -> Self {
        Self {
            set: set.clone(),
            active,
        }
    }
}
