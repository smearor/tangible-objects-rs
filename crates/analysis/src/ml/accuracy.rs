use serde::Serialize;
use std::fmt::Display;

/// Classification accuracy - a share of correct predictions, clamped to
/// `0.0..=1.0`.
///
/// `Option<Accuracy>` is `None` when evaluation was impossible (no held-out
/// samples, untrainable model); a `NaN` input clamps to `0.0` rather than
/// propagating.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct Accuracy(pub f64);

impl Accuracy {
    /// Creates an accuracy value, clamped to `0.0..=1.0`.
    pub fn new(value: f64) -> Self {
        if value.is_nan() {
            return Self(0.0);
        }
        Self(value.clamp(0.0, 1.0))
    }

    /// The raw fraction (0.0–1.0).
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Computes accuracy from correct/total predictions; `None` for an
    /// empty evaluation set.
    pub fn of(correct: usize, total: usize) -> Option<Self> {
        if total == 0 {
            return None;
        }
        Some(Self::new(correct as f64 / total as f64))
    }
}

impl Display for Accuracy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}%", self.0 * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Accuracy;

    #[test]
    fn clamps_out_of_range_values() {
        assert_eq!(Accuracy::new(1.7).as_f64(), 1.0);
        assert_eq!(Accuracy::new(-0.5).as_f64(), 0.0);
        assert_eq!(Accuracy::new(f64::NAN).as_f64(), 0.0);
    }

    #[test]
    fn computes_share_and_empty_is_none() {
        assert_eq!(Accuracy::of(3, 4).map(|a| a.as_f64()), Some(0.75));
        assert!(Accuracy::of(0, 0).is_none());
    }
}
