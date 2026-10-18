use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

/// Touch indices assigned to a figure by missing-pin completion -
/// pins predicted from the matched geometry that did not satisfy the
/// fingerprint themselves (TRACKING_STABILIZATION.md).
///
/// The *result* of claiming - distinct from [`crate::ClaimPool`],
/// which is the input search space the claims are drawn from. Empty
/// when completion did not run or found nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClaimedIndices(pub Vec<usize>);

impl ClaimedIndices {
    /// The index slice.
    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

impl From<Vec<usize>> for ClaimedIndices {
    /// An index vector IS a claimed set.
    fn from(indices: Vec<usize>) -> Self {
        Self(indices)
    }
}

impl<'a> IntoIterator for &'a ClaimedIndices {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    /// Iterating claimed indices is iterating the slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for ClaimedIndices {
    type Target = [usize];

    /// Claimed sets deref to their index slice.
    fn deref(&self) -> &[usize] {
        &self.0
    }
}
