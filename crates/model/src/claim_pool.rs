use std::ops::Deref;

/// Touch indices still claimable for missing-pin completion - the
/// search space [`crate::TangiblePattern::predict_missing`] matches
/// predicted pin positions against.
///
/// Distinct from [`crate::Combination`]: the pool never describes the
/// candidate itself, only the contacts a partial match may still
/// claim.
#[derive(Debug, Clone, Copy)]
pub struct ClaimPool<'a>(pub &'a [usize]);

impl<'a> ClaimPool<'a> {
    /// Wraps a pool slice.
    pub const fn new(indices: &'a [usize]) -> Self {
        Self(indices)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [usize] {
        self.0
    }
}

impl<'a> From<&'a [usize]> for ClaimPool<'a> {
    fn from(indices: &'a [usize]) -> Self {
        Self(indices)
    }
}

impl<'a> From<&'a Vec<usize>> for ClaimPool<'a> {
    fn from(indices: &'a Vec<usize>) -> Self {
        Self(indices)
    }
}

impl<'a, const N: usize> From<&'a [usize; N]> for ClaimPool<'a> {
    fn from(indices: &'a [usize; N]) -> Self {
        Self(indices)
    }
}

impl<'a> IntoIterator for ClaimPool<'a> {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    /// Iterating a pool is iterating its indices.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> Deref for ClaimPool<'a> {
    type Target = [usize];

    /// Pools view their index slice transparently.
    fn deref(&self) -> &[usize] {
        self.0
    }
}
