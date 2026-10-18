use std::ops::Deref;

/// Touch indices selected as one candidate combination - the contacts
/// under test as a figure's pins.
///
/// Produced by `combinations()` enumeration and stored as
/// `PatternMatch::touch_indices`. Distinct from [`crate::ClaimPool`]:
/// a combination IS the candidate, a pool is the claimable search
/// space around it.
#[derive(Debug, Clone, Copy)]
pub struct Combination<'a>(pub &'a [usize]);

impl<'a> Combination<'a> {
    /// Wraps a combination slice.
    pub const fn new(indices: &'a [usize]) -> Self {
        Self(indices)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [usize] {
        self.0
    }
}

impl<'a> From<&'a [usize]> for Combination<'a> {
    fn from(indices: &'a [usize]) -> Self {
        Self(indices)
    }
}

impl<'a> From<&'a Vec<usize>> for Combination<'a> {
    fn from(indices: &'a Vec<usize>) -> Self {
        Self(indices)
    }
}

impl<'a, const N: usize> From<&'a [usize; N]> for Combination<'a> {
    fn from(indices: &'a [usize; N]) -> Self {
        Self(indices)
    }
}

impl<'a> IntoIterator for Combination<'a> {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    /// Iterating a combination is iterating its indices.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> Deref for Combination<'a> {
    type Target = [usize];

    /// Combinations view their index slice transparently.
    fn deref(&self) -> &[usize] {
        self.0
    }
}
