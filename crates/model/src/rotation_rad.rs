use crate::SmoothingAlpha;
use crate::Vec2;
use serde::Deserialize;
use serde::Serialize;
use std::f64::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;

/// A rotation angle in radians.
///
/// Wraps the plain `f64` so angles cannot be confused with distances or
/// other scalar values flowing through the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RotationRad(pub f64);

impl RotationRad {
    /// No rotation.
    pub const ZERO: Self = Self(0.0);

    /// Creates a rotation from radians.
    pub const fn new(radians: f64) -> Self {
        Self(radians)
    }

    /// The angle in radians.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Whether the rotation is zero (no turn).
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    /// The angle converted to degrees.
    pub fn to_degrees(self) -> f64 {
        self.0.to_degrees()
    }

    /// `(sin, cos)` of the angle - trigonomtrie for transforms and
    /// geometry (the `f64::sin_cos` delegate).
    pub fn sin_cos(self) -> (f64, f64) {
        self.0.sin_cos()
    }

    /// The interior angle between two consecutive edge directions
    /// of a positive-area polygon: `PI - signed turn` from
    /// `previous` to `next` (sharp corners < ~PI/2).
    pub fn interior<U>(previous: Vec2<U>, next: Vec2<U>) -> Self {
        Self(
            PI - previous
                .cross::<f64>(&next)
                .atan2(previous.dot::<f64>(&next)),
        )
    }

    /// The shortest signed arc from `self` to `to`, in [-π, π].
    ///
    /// The ±PI wrap is crossed instead of going the long way around;
    /// the sign tells the turning direction.
    pub fn signed_delta_to(self, to: RotationRad) -> RotationRad {
        let mut delta = to.0 - self.0;
        while delta > PI {
            delta -= 2.0 * PI;
        }
        while delta < -PI {
            delta += 2.0 * PI;
        }
        RotationRad(delta)
    }

    /// Wraparound-aware EMA blend - `old + alpha * diff(new, old)`,
    /// where `diff` is the shortest signed arc between the angles
    /// (modulo 2π). `alpha` is the weight of `new`.
    pub fn ema(old: RotationRad, new: RotationRad, alpha: SmoothingAlpha) -> RotationRad {
        Self(old.0 + alpha.as_f64() * old.signed_delta_to(new).0)
    }

    /// The direction normalized to an undirected axis in
    /// `[-pi/2, pi/2)` - `self` and `self + pi` (the antipodal
    /// direction) share an axis, so heading-ambiguous footprints
    /// like two-pin dumbbells fold onto one stable value.
    pub fn axis(&self) -> RotationRad {
        RotationRad(self.0.rem_euclid(PI) - PI / 2.0)
    }
}

impl From<f64> for RotationRad {
    fn from(radians: f64) -> Self {
        Self(radians)
    }
}

impl From<RotationRad> for f64 {
    fn from(rotation: RotationRad) -> Self {
        rotation.0
    }
}

impl Add for RotationRad {
    type Output = RotationRad;

    /// Composes two rotations.
    fn add(self, other: RotationRad) -> RotationRad {
        RotationRad(self.0 + other.0)
    }
}

impl AddAssign for RotationRad {
    /// Adds a rotation in place.
    fn add_assign(&mut self, other: RotationRad) {
        self.0 += other.0;
    }
}

impl Sub for RotationRad {
    type Output = RotationRad;

    /// The signed difference between two rotations.
    fn sub(self, other: RotationRad) -> RotationRad {
        RotationRad(self.0 - other.0)
    }
}

impl Neg for RotationRad {
    type Output = RotationRad;

    /// The inverse rotation.
    fn neg(self) -> RotationRad {
        RotationRad(-self.0)
    }
}

#[cfg(feature = "async-graphql")]
async_graphql::scalar!(RotationRad);

#[cfg(test)]
mod tests {
    use super::RotationRad;
    use crate::SmoothingAlpha;
    use std::f64::consts::PI;

    /// `alpha = 1` makes the blend pure `new` - history has no weight.
    #[test]
    fn ema_full_weight_is_new() {
        let result = RotationRad::ema(
            RotationRad::new(1.0),
            RotationRad::new(2.0),
            SmoothingAlpha::new(1.0),
        );
        assert_eq!(result.as_f64(), 2.0);
    }

    /// `alpha = 0` keeps `old` - a frozen pose ignores measurement.
    #[test]
    fn ema_zero_weight_is_old() {
        let result = RotationRad::ema(
            RotationRad::new(1.0),
            RotationRad::new(2.0),
            SmoothingAlpha::new(0.0),
        );
        assert_eq!(result.as_f64(), 1.0);
    }

    /// The shortest arc crosses ±π upward: `3.0 → -3.0` passes over
    /// π, not back through 0.
    #[test]
    fn ema_crosses_pi_boundary_shortest_way() {
        let result = RotationRad::ema(
            RotationRad::new(3.0),
            RotationRad::new(-3.0),
            SmoothingAlpha::new(0.5),
        );
        assert!(result.as_f64() > 3.0);
        assert!(result.as_f64() <= PI + 1e-9);
    }

    /// Within ±PI the arc is the plain difference.
    #[test]
    fn signed_delta_inside_bounds() {
        let delta = RotationRad::new(1.0).signed_delta_to(RotationRad::new(2.0));
        assert_eq!(delta.as_f64(), 1.0);
    }

    /// `3.0 -> -3.0` wraps across +PI (delta ~= +0.28), not -6 the
    /// long way.
    #[test]
    fn signed_delta_wraps_shortest_way() {
        let delta = RotationRad::new(3.0).signed_delta_to(RotationRad::new(-3.0));
        assert!((delta.as_f64() - (2.0 * PI - 6.0)).abs() < 1e-9);
    }

    /// The mirrored crossing reports the negative direction.
    #[test]
    fn signed_delta_wraps_negative_side() {
        let delta = RotationRad::new(-3.0).signed_delta_to(RotationRad::new(3.0));
        assert!((delta.as_f64() - (6.0 - 2.0 * PI)).abs() < 1e-9);
    }
    #[test]
    fn interior_angle_of_a_right_turn() {
        let angle = RotationRad::interior(
            crate::Vec2::<crate::Millimeters>::new(1.0, 0.0),
            crate::Vec2::new(0.0, 1.0),
        );
        assert!((angle.as_f64() - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    }

    /// `axis` folds `a` and `a + pi` onto the same undirected angle.
    #[test]
    fn axis_ignores_direction() {
        let (a, b) = (RotationRad::new(0.3), RotationRad::new(0.3 + PI));
        assert!((a.axis().as_f64() - b.axis().as_f64()).abs() < 1e-9);
        assert!((a.axis().as_f64() - (0.3 - PI / 2.0)).abs() < 1e-9);
    }

    /// `axis` keeps every direction inside `[-pi/2, pi/2)`, even for
    /// non-normalized inputs - `rem_euclid` wraps full turns first.
    #[test]
    fn axis_stays_in_half_turn_range() {
        for angle in [
            0.0,
            1.0,
            -1.0,
            PI / 2.0,
            -PI / 2.0,
            PI,
            -PI,
            4.0,
            -4.0,
            42.0,
        ] {
            let axis = RotationRad::new(angle).axis().as_f64();
            assert!(
                (-PI / 2.0..PI / 2.0).contains(&axis),
                "axis({angle}) = {axis} outside [-pi/2, pi/2)"
            );
        }
    }

    /// `axis` measures the line's tilt off vertical: `0` and `pi`
    /// (horizontal) fold to `-pi/2`, `+-pi/2` (vertical) folds to `0`.
    #[test]
    fn axis_boundary_values() {
        assert!((RotationRad::new(0.0).axis().as_f64() + PI / 2.0).abs() < 1e-9);
        assert!(RotationRad::new(PI / 2.0).axis().as_f64().abs() < 1e-9);
        assert!(RotationRad::new(-PI / 2.0).axis().as_f64().abs() < 1e-9);
        assert!((RotationRad::new(PI).axis().as_f64() + PI / 2.0).abs() < 1e-9);
    }
}
