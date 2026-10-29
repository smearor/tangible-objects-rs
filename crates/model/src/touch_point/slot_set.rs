use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::ops::Deref;
use std::ops::DerefMut;

use super::TouchPointSlot;

/// A set of slot identities - which slots a stage binds, emits,
/// suppresses or matched this frame.
///
/// The collection counterpart of [`TouchPointSlot`]: `matched_slots` /
/// `slots_of` of a match, a stage's `suppressed`/`emitted` mask, or a
/// tracker's bound-slot map. The parallel of [`crate::RefIndices`]
/// for slot identities.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SlotSet(pub HashSet<TouchPointSlot>);

impl FromIterator<TouchPointSlot> for SlotSet {
    /// Collecting slots yields a slot set.
    fn from_iter<T: IntoIterator<Item = TouchPointSlot>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<HashSet<TouchPointSlot>> for SlotSet {
    /// A hash set IS a slot set.
    fn from(slots: HashSet<TouchPointSlot>) -> Self {
        Self(slots)
    }
}

impl<const N: usize> From<[TouchPointSlot; N]> for SlotSet {
    /// An array of slots IS a slot set - the `HashSet::from` idiom.
    fn from(slots: [TouchPointSlot; N]) -> Self {
        slots.into_iter().collect()
    }
}

impl Deref for SlotSet {
    type Target = HashSet<TouchPointSlot>;

    /// Slot sets deref to the underlying `HashSet` - `contains`,
    /// `len`, `iter`, `extend` stay available.
    fn deref(&self) -> &HashSet<TouchPointSlot> {
        &self.0
    }
}

impl DerefMut for SlotSet {
    /// Mutable access to the underlying `HashSet`.
    fn deref_mut(&mut self) -> &mut HashSet<TouchPointSlot> {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a SlotSet {
    type Item = &'a TouchPointSlot;
    type IntoIter = Iter<'a, TouchPointSlot>;

    /// Iterating a slot set is iterating its `HashSet`.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
