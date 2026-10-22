use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;

/// A length or coordinate in millimeters on the touch surface.
///
/// Wraps the plain `f64` so millimeter values cannot be confused with raw
/// device units or screen pixels. Also serves as the unit marker for
/// `Vec2<Millimeters>`. Serializes as a bare number so profile files stay
/// compatible.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct Millimeters(pub f64);

impl Millimeters {
    /// Creates a millimeter value.
    pub const fn new(millimeters: f64) -> Self {
        Self(millimeters)
    }

    /// The numeric value in millimeters.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Absolute value.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Total ordering comparison (no NaN pitfalls when sorting).
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Add for Millimeters {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Millimeters {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Millimeters {
    type Output = Self;

    fn mul(self, factor: f64) -> Self {
        Self(self.0 * factor)
    }
}

impl Sum for Millimeters {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0.0), Add::add)
    }
}

impl From<f64> for Millimeters {
    fn from(millimeters: f64) -> Self {
        Self(millimeters)
    }
}

impl From<Millimeters> for f64 {
    fn from(millimeters: Millimeters) -> Self {
        millimeters.0
    }
}

impl Display for Millimeters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(Millimeters);
