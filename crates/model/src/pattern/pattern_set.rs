use std::ops::Deref;

use super::PatternBuf;
use super::TangiblePattern;

/// The registered figure patterns - a borrowed view, not an
/// allocation.
///
/// Matching and analysis APIs take `PatternSet` so "the pattern
/// registry" is a named concept; `Engine` keeps the owned `Vec`.
#[derive(Debug, Clone, Copy)]
pub struct PatternSet<'a>(pub &'a [TangiblePattern]);

impl<'a> PatternSet<'a> {
    /// Wraps a pattern slice.
    pub const fn new(patterns: &'a [TangiblePattern]) -> Self {
        Self(patterns)
    }

    /// The wrapped slice.
    pub const fn as_slice(self) -> &'a [TangiblePattern] {
        self.0
    }
}

impl<'a> From<&'a [TangiblePattern]> for PatternSet<'a> {
    fn from(patterns: &'a [TangiblePattern]) -> Self {
        Self(patterns)
    }
}

impl<'a> From<&'a Vec<TangiblePattern>> for PatternSet<'a> {
    fn from(patterns: &'a Vec<TangiblePattern>) -> Self {
        Self(patterns)
    }
}

impl<'a> From<&'a PatternBuf> for PatternSet<'a> {
    /// Borrowed view of an owned pattern set.
    fn from(patterns: &'a PatternBuf) -> Self {
        Self(&patterns.0)
    }
}

impl<'a, const N: usize> From<&'a [TangiblePattern; N]> for PatternSet<'a> {
    fn from(patterns: &'a [TangiblePattern; N]) -> Self {
        Self(patterns)
    }
}

impl<'a> IntoIterator for PatternSet<'a> {
    type Item = &'a TangiblePattern;
    type IntoIter = std::slice::Iter<'a, TangiblePattern>;

    /// Iterating a set is iterating its slice.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> Deref for PatternSet<'a> {
    type Target = [TangiblePattern];

    /// Pattern sets view patterns transparently.
    fn deref(&self) -> &[TangiblePattern] {
        self.0
    }
}
