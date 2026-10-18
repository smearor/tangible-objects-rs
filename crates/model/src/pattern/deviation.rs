use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;

/// Absolute deviation `|measured - reference|` of one fingerprint
/// index - a delta in millimeters, not a distance.
///
/// A deviation answers "how far off is this side" while a distance
/// answers "how long is this side" - same unit, different semantics.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Deviation(pub Millimeters);

impl Deviation {
    /// Wraps a millimeter deviation.
    pub const fn new(deviation: Millimeters) -> Self {
        Self(deviation)
    }

    /// The wrapped millimeters.
    pub const fn as_millimeters(self) -> Millimeters {
        self.0
    }

    /// The deviation as a plain number.
    pub fn as_f64(self) -> f64 {
        self.0.as_f64()
    }

    /// Total-order comparison (f64 has no `Ord`).
    pub fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<Millimeters> for Deviation {
    /// A millimeter delta IS a deviation.
    fn from(deviation: Millimeters) -> Self {
        Self(deviation)
    }
}
