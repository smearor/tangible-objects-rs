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
pub struct Field {
    /// Cell name (e.g. `"a1"` on a chess board).
    pub name: FieldName,
    /// Cell bounds in the set's coordinate space (screen mm, or
    /// window-relative / normalized when the set has an anchor).
    #[serde(flatten)]
    pub region: Region,
}
