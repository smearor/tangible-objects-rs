use std::ops::Deref;
use std::ops::DerefMut;

use super::PatternId;
use super::PatternSet;
use super::TangiblePattern;
use crate::PointSet;

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

    /// The pattern with `id`, or `None` - registration/lookup by ID.
    pub fn by_id(&self, id: PatternId) -> Option<&TangiblePattern> {
        self.iter().find(|pattern| pattern.id == id)
    }

    /// Mutable [`Self::by_id`] - runtime profile updates.
    pub fn by_id_mut(&mut self, id: PatternId) -> Option<&mut TangiblePattern> {
        self.iter_mut().find(|pattern| pattern.id == id)
    }

    /// First pattern in the set matching `points`, skipping `except`
    /// (the pattern an update replaces - the fingerprint collision
    /// check of runtime registration).
    pub fn find_matching(
        &self,
        points: PointSet<'_>,
        except: Option<PatternId>,
    ) -> Option<&TangiblePattern> {
        self.iter()
            .filter(|existing| Some(existing.id) != except)
            .find(|existing| existing.matches(points))
    }

    /// The next assignable pattern ID - `max(id) + 1`, `0` on an
    /// empty set. Seeds engine-side `next_pattern_id` counters for
    /// runtime registration.
    pub fn next_id(&self) -> PatternId {
        self.iter()
            .map(|pattern| pattern.id)
            .max()
            .map_or(PatternId::new(0), |id| id.next())
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
