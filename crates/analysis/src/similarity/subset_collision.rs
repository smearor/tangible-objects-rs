use tangible_objects_model::ContactCount;
use tangible_objects_model::Millimeters;

/// The closest same-size fingerprint collision found between two
/// patterns - the geometric phantom surface of the pair.
///
/// `contact_count == num_points` means even the exact fingerprints
/// collide; lower counts mean only partial subsets approach each other.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SubsetCollision {
    /// Number of contacts kept in the closest subset pairing.
    pub(crate) contact_count: ContactCount,
    /// Smallest achievable maximum deviation across subset pairings.
    pub(crate) min_max_deviation: Millimeters,
}
