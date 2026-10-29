use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use crate::PinCount;
use crate::RefIndex;
use crate::RefPoints;

/// A set of reference pin indices (pin identities - ROTATION.md) -
/// the kept subset of a partial match, the skip set of a partial
/// reference, or a claim's bound pin.
///
/// Owned counterpart of `&[RefIndex]` function parameters; home of
/// the projections every pin-set consumer used to spell out inline.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RefIndices(pub Vec<RefIndex>);

impl RefIndices {
    /// All pins of a `num_points` pattern - the full set an exact
    /// match's correspondence covers.
    pub fn all(num_points: PinCount) -> Self {
        (0..num_points.as_usize()).map(RefIndex::new).collect()
    }

    /// The reference positions of these pins -
    /// `ref_points[pin]` for every pin in the set.
    pub fn points_of(&self, ref_points: &RefPoints) -> RefPoints {
        self.iter()
            .map(|&index| ref_points[index.as_usize()])
            .collect::<Vec<_>>()
            .into()
    }

    /// The pins **not** in this set - the missing pins the
    /// completion path predicts (TRACKING_STABILIZATION.md).
    pub fn complement(&self, num_points: PinCount) -> RefIndices {
        (0..num_points.as_usize())
            .filter(|index| !self.contains(&RefIndex::new(*index)))
            .map(RefIndex::new)
            .collect()
    }
}

impl From<Vec<RefIndex>> for RefIndices {
    /// A pin-index vector IS a ref-indices set.
    fn from(indices: Vec<RefIndex>) -> Self {
        Self(indices)
    }
}

impl FromIterator<RefIndex> for RefIndices {
    /// Collecting pin indices yields the set.
    fn from_iter<T: IntoIterator<Item = RefIndex>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for RefIndices {
    type Item = RefIndex;
    type IntoIter = std::vec::IntoIter<RefIndex>;

    /// Iterating an owned set yields its pins.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for RefIndices {
    type Target = [RefIndex];

    /// A ref-indices set derefs to its pin slice - `&[RefIndex]`
    /// parameters stay slice-compatible.
    fn deref(&self) -> &[RefIndex] {
        &self.0
    }
}
