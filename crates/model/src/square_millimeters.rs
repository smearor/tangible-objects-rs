use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;

/// An area in square millimeters on the touch surface.
///
/// Wraps the plain `f64` so surface areas cannot be confused with linear
/// millimeters or pixel areas: `Vec2<Millimeters>` geometry produces it for
/// shoelace areas and size thresholds. Serializes as a bare number like
/// [`Millimeters`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct SquareMillimeters(pub f64);

impl SquareMillimeters {
    /// Empty area.
    pub const ZERO: Self = Self(0.0);

    /// Creates a square-millimeter value.
    pub const fn new(square_millimeters: f64) -> Self {
        Self(square_millimeters)
    }

    /// The numeric value in square millimeters.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Absolute value.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// The smaller of two values (`f64::min` semantics for NaN inputs).
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// The larger of two values (`f64::max` semantics for NaN inputs).
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Total ordering comparison (no NaN pitfalls when sorting).
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Add for SquareMillimeters {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for SquareMillimeters {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for SquareMillimeters {
    type Output = Self;

    fn mul(self, factor: f64) -> Self {
        Self(self.0 * factor)
    }
}

impl Sum for SquareMillimeters {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl From<f64> for SquareMillimeters {
    fn from(square_millimeters: f64) -> Self {
        Self(square_millimeters)
    }
}

impl From<SquareMillimeters> for f64 {
    fn from(square_millimeters: SquareMillimeters) -> Self {
        square_millimeters.0
    }
}

impl Display for SquareMillimeters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(SquareMillimeters);
