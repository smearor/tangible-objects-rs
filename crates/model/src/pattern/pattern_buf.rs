use std::ops::Deref;
use std::ops::DerefMut;

use super::PatternSet;
use super::TangiblePattern;

/// Owned set of registered figure patterns - the materialized
/// counterpart of the borrowed [`PatternSet`] view.
///
/// Profile loading and the engine store a `PatternBuf`; everything
/// consuming patterns works on the `PatternSet` view via
/// [`Self::as_pattern_set`].
#[derive(Debug, Clone, Default)]
pub struct PatternBuf(pub Vec<TangiblePattern>);

impl PatternBuf {
    /// Borrowed view of this pattern set.
    pub fn as_pattern_set(&self) -> PatternSet<'_> {
        PatternSet::new(&self.0)
    }
}

impl From<Vec<TangiblePattern>> for PatternBuf {
    /// A pattern vector IS an owned pattern set.
    fn from(patterns: Vec<TangiblePattern>) -> Self {
        Self(patterns)
    }
}

impl From<PatternBuf> for Vec<TangiblePattern> {
    /// Unwraps the owned pattern vector.
    fn from(patterns: PatternBuf) -> Self {
        patterns.0
    }
}

impl Deref for PatternBuf {
    type Target = Vec<TangiblePattern>;

    /// Owned pattern sets deref to their vector - `push`/`retain` for
    /// runtime registration live on the owned side.
    fn deref(&self) -> &Vec<TangiblePattern> {
        &self.0
    }
}

impl DerefMut for PatternBuf {
    /// Mutable access registers or removes patterns.
    fn deref_mut(&mut self) -> &mut Vec<TangiblePattern> {
        &mut self.0
    }
}
