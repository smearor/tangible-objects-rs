use crate::Distances;
use crate::RefIndices;

/// Reference subset with its reduced fingerprint - the pins that remain
/// after dropping `skips` for partial matching.
///
/// `kept_indices` preserves the reference ordering; the complement are
/// the missing pins, used by missing-pin completion to predict their
/// expected positions.
#[derive(Debug, Clone)]
pub struct PartialReference {
    /// Pin identities of the kept reference points, in reference order.
    pub kept_indices: RefIndices,
    /// Sorted pairwise distances of the kept subset - the reduced
    /// fingerprint the measured points are compared against.
    pub distances: Distances,
}
