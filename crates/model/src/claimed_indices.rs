use serde::Deserialize;
use serde::Serialize;

use crate::Claim;

/// Completion claims assigned to a figure - pins predicted from the
/// matched geometry that did not satisfy the fingerprint themselves
/// (TRACKING_STABILIZATION.md). The `Claim`'s ref index lets the
/// tracker bind a claimed slot to its reference pin (ROTATION.md).
///
/// The *result* of claiming - distinct from [`crate::ClaimPool`],
/// which is the input search space the claims are drawn from. Empty
/// when completion did not run or found nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClaimedIndices(pub Vec<Claim>);

impl ClaimedIndices {
    /// The claim slice.
    pub fn as_slice(&self) -> &[Claim] {
        &self.0
    }

    /// Number of claimed pins.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no pins were claimed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The claimed touch indices.
    pub fn touch_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().map(|claim| claim.touch_index)
    }

    /// Whether `index` is a claimed touch index.
    pub fn contains_touch(&self, index: usize) -> bool {
        self.touch_indices().any(|touch| touch == index)
    }
}

impl From<Vec<Claim>> for ClaimedIndices {
    /// A claim vector IS a claimed set.
    fn from(indices: Vec<Claim>) -> Self {
        Self(indices)
    }
}

impl<'a> IntoIterator for &'a ClaimedIndices {
    type Item = &'a Claim;
    type IntoIter = std::slice::Iter<'a, Claim>;

    /// Iterating claimed indices is iterating the claim slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
