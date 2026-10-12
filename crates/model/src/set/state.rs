use super::TangibleSet;
use serde::Deserialize;
use serde::Serialize;

/// A [`TangibleSet`] definition joined with its live activation state.
///
/// The `active` flag is runtime state derived by the [`SetTracker`] and not
/// part of the persisted set definition; this type bundles both for
/// interface-facing queries. Serializes flat as `{id, name, mandatory,
/// additional, active}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct TangibleSetState {
    /// The set definition; serialized inline via `#[serde(flatten)]`.
    #[serde(flatten)]
    pub set: TangibleSet,
    /// Whether the set is currently active on the surface.
    pub active: bool,
}

impl From<(&TangibleSet, bool)> for TangibleSetState {
    fn from((set, active): (&TangibleSet, bool)) -> Self {
        Self {
            set: set.clone(),
            active,
        }
    }
}
