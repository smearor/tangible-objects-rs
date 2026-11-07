use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

/// A coordinate or extent in screen pixels.
///
/// Wraps the plain `f64` so pixel values cannot be confused with millimeter
/// positions. Also serves as the unit marker for `Vec2<Pixels>`.
/// Serializes as a bare number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pixels(pub f64);

impl Pixels {
    /// Creates a pixel value.
    pub const fn new(pixels: f64) -> Self {
        Self(pixels)
    }

    /// The numeric value in pixels.
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

    /// Clamps the value into the inclusive `[min, max]` range.
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }
}

impl Add for Pixels {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Pixels {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Pixels {
    type Output = Self;

    fn mul(self, factor: f64) -> Self {
        Self(self.0 * factor)
    }
}

impl Mul<Pixels> for f64 {
    type Output = Pixels;

    fn mul(self, rhs: Pixels) -> Pixels {
        rhs * self
    }
}

impl Div<f64> for Pixels {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self(self.0 / divisor)
    }
}

impl Neg for Pixels {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Div<Pixels> for Pixels {
    type Output = f64;

    /// Same-unit ratio - the dimensionless scalar.
    fn div(self, rhs: Pixels) -> f64 {
        self.0 / rhs.0
    }
}

impl Sum for Pixels {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0.0), Add::add)
    }
}

impl From<f64> for Pixels {
    fn from(pixels: f64) -> Self {
        Self(pixels)
    }
}

impl From<Pixels> for f64 {
    fn from(pixels: Pixels) -> Self {
        pixels.0
    }
}

impl Display for Pixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Pixels;

    #[test]
    fn ops_work() {
        let px = Pixels::new(4.0);
        assert_eq!((-px).as_f64(), -4.0);
        assert_eq!((px / 2.0).as_f64(), 2.0);
        assert_eq!((2.0 * px).as_f64(), 8.0);
        assert_eq!([px, px, px].into_iter().sum::<Pixels>().as_f64(), 12.0);
    }
}
