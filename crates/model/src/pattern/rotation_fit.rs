use crate::RefIndices;
use crate::RotationRad;

/// A correspondence-fitted rotation (ROTATION.md): the solved angle
/// plus the pin assignment it was fitted under.
///
/// `correspondence[j]` is the *full* reference pin index of
/// `measured[j]` - the positional mapping of the winning
/// correspondence enumeration, resolved against the kept-subset
/// indexing back into full-pin space.
#[derive(Debug, Clone)]
pub struct RotationFit {
    /// The fitted rotation from reference frame to measured frame.
    pub angle: RotationRad,
    /// The pin correspondence the angle was fitted under -
    /// `correspondence[j]` names the reference pin of measured
    /// position `j`.
    pub correspondence: RefIndices,
}
