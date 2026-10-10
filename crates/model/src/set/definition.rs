use serde::Deserialize;
use serde::Serialize;

use super::TangibleSetId;
use crate::PatternId;

/// Definition of a Tangible Object Set (TOS).
///
/// A set activates when **all** `mandatory` member patterns are present on
/// the touch surface and stays active while **at least one** mandatory
/// member remains (activation hysteresis, see `TANGIBLE_OBJECT_SET.md`).
/// `additional` members belong to the set but never affect activation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TangibleSet {
    /// Unique identifier of the set.
    pub id: TangibleSetId,
    /// Human-readable name of the set.
    pub name: String,
    /// Patterns that must all be present to activate the set.
    pub mandatory: Vec<PatternId>,
    /// Optional member patterns; never affect activation.
    pub additional: Vec<PatternId>,
}

impl TangibleSet {
    /// Whether `pattern` belongs to this set (mandatory or additional).
    pub fn contains(&self, pattern: PatternId) -> bool {
        self.is_mandatory(pattern) || self.additional.contains(&pattern)
    }

    /// Whether `pattern` is a mandatory member of this set.
    pub fn is_mandatory(&self, pattern: PatternId) -> bool {
        self.mandatory.contains(&pattern)
    }
}
