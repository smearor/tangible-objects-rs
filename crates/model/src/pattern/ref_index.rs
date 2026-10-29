use serde::Deserialize;
use serde::Serialize;

/// Index into a pattern's `ref_points` - **pin identity**, not a
/// positional index (ROTATION.md). Distinct from touch indices (frame
/// positions) and from sliced positions inside a partial-subset solve:
/// a `RefIndex` names the same pin across subsets, claims and the
/// tracker's `pin_map`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RefIndex(usize);

impl RefIndex {
    /// The raw index value - for `ref_points[ref.as_usize()]` lookups.
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// The raw index value - for `ref_points[ref.as_usize()]` lookups.
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for RefIndex {
    /// A raw pin index IS a `RefIndex`.
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<RefIndex> for usize {
    /// The raw index value.
    fn from(index: RefIndex) -> Self {
        index.0
    }
}
