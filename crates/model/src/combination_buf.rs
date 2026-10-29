use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use crate::Combination;
use crate::RefIndex;
use crate::RefIndices;

/// Owned touch indices of a winning candidate - the materialized
/// counterpart of the borrowed [`Combination`] view.
///
/// `PatternMatch` and `CandidateReport` store the indices that formed
/// the figure; matching APIs work on the `Combination` view.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CombinationBuf(pub Vec<usize>);

impl CombinationBuf {
    /// Borrowed view of this combination.
    pub fn as_combination(&self) -> Combination<'_> {
        Combination::new(&self.0)
    }

    /// `(touch_index, ref)` pairs of this combination under a pin
    /// correspondence - position `i` pairs `self[i]` with `refs[i]`;
    /// positions without a pin are skipped (a partial solve may bind
    /// fewer than `len` positions).
    pub fn corresponding<'a>(
        &'a self,
        refs: &'a RefIndices,
    ) -> impl Iterator<Item = (usize, RefIndex)> + 'a {
        self.iter()
            .enumerate()
            .filter_map(|(i, &touch_index)| refs.get(i).map(|&ref_index| (touch_index, ref_index)))
    }

    /// The same indices reinterpreted as pin identities (ROTATION.md):
    /// `skip_combinations` enumerates reference positions, so its
    /// combinations are ref indices, not touch indices.
    pub fn ref_indices(&self) -> RefIndices {
        self.0.iter().map(|&index| index.into()).collect()
    }
}

impl<'a> From<Combination<'a>> for CombinationBuf {
    /// Materializes a borrowed combination.
    fn from(combo: Combination<'a>) -> Self {
        Self(combo.as_slice().to_vec())
    }
}

impl From<Vec<usize>> for CombinationBuf {
    /// An index vector IS an owned combination.
    fn from(indices: Vec<usize>) -> Self {
        Self(indices)
    }
}

impl FromIterator<usize> for CombinationBuf {
    /// Collecting indices yields a combination.
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<usize> for CombinationBuf {
    /// Extending adds indices to the combination - `unzip` support.
    fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl From<&[usize]> for CombinationBuf {
    /// Materializes an index slice.
    fn from(indices: &[usize]) -> Self {
        Self(indices.to_vec())
    }
}

impl<'a> IntoIterator for &'a CombinationBuf {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    /// Iterating a buffer is iterating its indices.
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for CombinationBuf {
    type Target = [usize];

    /// Owned combinations deref to their index slice.
    fn deref(&self) -> &[usize] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::CombinationBuf;
    use crate::RefIndex;
    use crate::RefIndices;

    /// `corresponding` pairs combo positions with refs index-wise.
    #[test]
    fn corresponding_pairs_positions_with_refs() {
        let combo = CombinationBuf::from(&[2, 4, 1][..]);
        let refs: RefIndices = vec![RefIndex::new(5), RefIndex::new(7), RefIndex::new(3)].into();
        let pairs: Vec<(usize, RefIndex)> = combo.corresponding(&refs).collect();
        assert_eq!(
            pairs,
            vec![
                (2, RefIndex::new(5)),
                (4, RefIndex::new(7)),
                (1, RefIndex::new(3))
            ]
        );
    }

    /// Positions beyond `refs` are skipped - a partial solve binds
    /// fewer than `len` positions.
    #[test]
    fn corresponding_skips_unmapped_positions() {
        let combo = CombinationBuf::from(&[0, 3, 7][..]);
        let refs: RefIndices = vec![RefIndex::new(1)].into();
        let pairs: Vec<(usize, RefIndex)> = combo.corresponding(&refs).collect();
        assert_eq!(pairs, vec![(0, RefIndex::new(1))]);
    }

    /// `ref_indices` reinterprets the index values as pin identities.
    #[test]
    fn ref_indices_reinterprets_positions() {
        let combo = CombinationBuf::from(&[0, 2, 5][..]);
        let values: Vec<usize> = combo.ref_indices().iter().map(|i| i.as_usize()).collect();
        assert_eq!(values, vec![0, 2, 5]);
    }
}
