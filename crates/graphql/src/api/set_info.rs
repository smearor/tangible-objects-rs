use async_graphql::ID;
use async_graphql::SimpleObject;
use tangible_objects_model::TangibleSet;

/// A Tangible Object Set definition with its current activation state.
#[derive(Debug, Clone, SimpleObject)]
pub struct SetInfo {
    /// Set identifier.
    pub id: ID,
    /// Human-readable set name.
    pub name: String,
    /// Pattern IDs that must all be present to activate the set.
    pub mandatory: Vec<u32>,
    /// Pattern IDs that may additionally join the active set.
    pub additional: Vec<u32>,
    /// Whether the set is currently active.
    pub active: bool,
}

impl From<(&TangibleSet, bool)> for SetInfo {
    fn from((set, active): (&TangibleSet, bool)) -> Self {
        Self {
            id: ID::from(set.id.to_string()),
            name: set.name.clone(),
            mandatory: set.mandatory.iter().map(|id| id.as_u32()).collect(),
            additional: set.additional.iter().map(|id| id.as_u32()).collect(),
            active,
        }
    }
}
