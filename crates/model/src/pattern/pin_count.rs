use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Sub;

use crate::Combinations;

/// Number of contact pins a figure must have - a structural property of
/// the pattern, not a measured observation.
///
/// Kept distinct from [`crate::ContactCount`]: `num_points` describes
/// what the figure *is*, `ContactCount` counts what the screen *sees*.
/// `Ord` derives `min`/`max` for subset-size arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PinCount(pub usize);

impl PinCount {
    /// Creates a pin count.
    pub const fn new(count: usize) -> Self {
        Self(count)
    }

    /// The raw count as `usize`.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// The raw count as `u32` (wire-facing fields like
    /// `ProfileInfo::num_points` in the runtime crate).
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    /// All `missing`-element pin subsets of `0..self` - the reference
    /// positions a partial fingerprint skips. Returned as positional
    /// combinations (ref indices, not touch indices).
    pub fn skip_combinations(self, missing: usize) -> Combinations {
        let mut result = Vec::new();
        let mut combo = Vec::with_capacity(missing);
        enumerate_subsets(self.0, missing, 0, &mut combo, &mut result);
        result.into()
    }
}

/// Subset backtracking of `skip_combinations`: `combo` accumulates
/// `missing` positions from `start..n`, recursing to `start + 1`.
fn enumerate_subsets(
    n: usize,
    missing: usize,
    start: usize,
    combo: &mut Vec<usize>,
    out: &mut Vec<crate::CombinationBuf>,
) {
    if combo.len() == missing {
        out.push(combo.clone().into());
        return;
    }
    for i in start..n {
        combo.push(i);
        enumerate_subsets(n, missing, i + 1, combo, out);
        combo.pop();
    }
}

impl From<usize> for PinCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl From<PinCount> for usize {
    fn from(count: PinCount) -> Self {
        count.0
    }
}

impl PartialEq<usize> for PinCount {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<usize> for PinCount {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<PinCount> for usize {
    fn eq(&self, other: &PinCount) -> bool {
        *self == other.0
    }
}

impl PartialOrd<PinCount> for usize {
    fn partial_cmp(&self, other: &PinCount) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Sub<usize> for PinCount {
    type Output = usize;

    /// A pin count minus pins to skip yields a plain index-space count.
    fn sub(self, rhs: usize) -> usize {
        self.0 - rhs
    }
}

impl Display for PinCount {
    /// Writes the raw pin count.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "zbus")]
impl zvariant::Type for PinCount {
    /// Marshals as `u64` like the backing `usize`.
    const SIGNATURE: &'static zvariant::Signature = <u64 as zvariant::Type>::SIGNATURE;
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(PinCount);

#[cfg(test)]
mod tests {
    use super::PinCount;

    /// C(4, 1) enumerates the four singleton skips.
    #[test]
    fn skip_combinations_one_of_four() {
        let skips = PinCount::new(4).skip_combinations(1);
        let combos: Vec<&[usize]> = skips
            .iter()
            .map(|c| c.as_combination().as_slice())
            .collect();
        assert_eq!(combos, vec![&[0], &[1], &[2], &[3]]);
    }

    /// C(4, 2) enumerates six distinct pairs.
    #[test]
    fn skip_combinations_two_of_four() {
        let skips = PinCount::new(4).skip_combinations(2);
        assert_eq!(skips.len(), 6);
        let combos: Vec<&[usize]> = skips
            .iter()
            .map(|c| c.as_combination().as_slice())
            .collect();
        assert_eq!(
            combos,
            vec![&[0, 1], &[0, 2], &[0, 3], &[1, 2], &[1, 3], &[2, 3]]
        );
    }

    /// C(n, n) is exactly the full-set combination.
    #[test]
    fn skip_combinations_all_is_full_set() {
        let skips = PinCount::new(3).skip_combinations(3);
        assert_eq!(skips.len(), 1);
        assert_eq!(skips[0].as_combination().as_slice(), &[0, 1, 2]);
    }

    /// C(n, 0) is exactly the empty combination.
    #[test]
    fn skip_combinations_zero_is_empty() {
        let skips = PinCount::new(4).skip_combinations(0);
        assert_eq!(skips.len(), 1);
        assert!(skips[0].as_combination().as_slice().is_empty());
    }
}
