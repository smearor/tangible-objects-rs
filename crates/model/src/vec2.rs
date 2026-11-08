use crate::Millimeters;
use crate::Pixels;
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

#[cfg(feature = "cynic")]
use crate::schema;

/// A 2D point or vector tagged with its coordinate unit `U`.
///
/// The phantom unit distinguishes positions in millimeters
/// (`Vec2<Millimeters>`), raw device units (`Vec2<RawAxisValue>`) and screen
/// pixels (`Vec2<Pixels>`) so they cannot be mixed up. Components stay plain
/// `f64`; the unit only labels which coordinate space the point lives in.
/// Serializes as a flat `{"x": .., "y": ..}` map; the unit marker is skipped.
/// With the `async-graphql` feature, `Vec2<Millimeters>` is exposed as the
/// GraphQL object type `Point`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(
    feature = "async-graphql",
    graphql(concrete(name = "Point", params(crate::Millimeters)))
)]
pub struct Vec2<U> {
    /// X coordinate in the unit `U`.
    pub x: f64,
    /// Y coordinate in the unit `U`.
    pub y: f64,
    /// Phantom marker tagging the coordinate unit; not serialized.
    #[serde(skip)]
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    unit: PhantomData<U>,
}

impl<U> Vec2<U> {
    /// The origin `(0, 0)` in the unit `U`.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        unit: PhantomData,
    };

    /// Creates a new 2D point in the unit `U`.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            unit: PhantomData,
        }
    }

    /// Euclidean distance to another point in the same unit.
    pub fn distance(&self, other: &Vec2<U>) -> U
    where
        U: From<f64>,
    {
        U::from(((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt())
    }

    /// Dot product `x·x + y·y` of two vectors in the same unit.
    ///
    /// The caller picks the product type `T` (like [`Vec2::distance`]'s
    /// unit return): `f64` for unit-free math or a unit type such as
    /// [`crate::SquareMillimeters`] for `Vec2<Millimeters>` products.
    pub fn dot<T: From<f64>>(&self, other: &Vec2<U>) -> T {
        T::from(self.x * other.x + self.y * other.y)
    }

    /// 2D determinant `x·y' − y·x'` (z component of the 3D cross
    /// product): positive when `other` turns left from `self`, negative
    /// right; zero for (anti-)parallel vectors.
    ///
    /// Same caller-chosen product type as [`Vec2::dot`].
    pub fn cross<T: From<f64>>(&self, other: &Vec2<U>) -> T {
        T::from(self.x * other.y - self.y * other.x)
    }

    /// Euclidean magnitude of the vector, in the unit `U`.
    pub fn magnitude(&self) -> U
    where
        U: From<f64>,
    {
        self.distance(&Vec2::ZERO)
    }

    /// The vector rotated 90° counterclockwise: `(-y, x)` - the
    /// perpendicular used for offset and normal computations.
    pub fn perpendicular(&self) -> Self {
        Vec2::new(-self.y, self.x)
    }

    /// The unit vector pointing the same way; a vector shorter
    /// than `1e-9` normalizes to itself.
    pub fn normalized(&self) -> Self {
        let magnitude = self.x.hypot(self.y).max(1e-9);
        Vec2::new(self.x / magnitude, self.y / magnitude)
    }

    /// Geometric centroid of a non-empty point set.
    ///
    /// Returns [`Vec2::new(0.0, 0.0)`] for an empty slice instead of panicking.
    pub fn centroid(points: &[Vec2<U>]) -> Self {
        let n = points.len() as f64;
        if n == 0.0 {
            return Vec2::new(0.0, 0.0);
        }
        Vec2::new(
            points.iter().map(|p| p.x).sum::<f64>() / n,
            points.iter().map(|p| p.y).sum::<f64>() / n,
        )
    }
}

impl Vec2<Millimeters> {
    /// Creates a point from typed millimeter components.
    pub fn from_mm(x: Millimeters, y: Millimeters) -> Self {
        Self::new(x.as_f64(), y.as_f64())
    }
}

impl Vec2<Pixels> {
    /// Creates a point from typed pixel components.
    pub fn from_px(x: Pixels, y: Pixels) -> Self {
        Self::new(x.as_f64(), y.as_f64())
    }
}

impl<U> Add for Vec2<U> {
    type Output = Self;

    /// Component-wise sum of two points in the same unit.
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<U> Sub for Vec2<U> {
    type Output = Self;

    /// Component-wise difference of two points in the same unit.
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<U> Neg for Vec2<U> {
    type Output = Self;

    /// The point mirrored at the origin.
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl<U> Mul<f64> for Vec2<U> {
    type Output = Self;

    /// The vector scaled by a dimensionless factor.
    fn mul(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl<U> Mul<Vec2<U>> for f64 {
    type Output = Vec2<U>;

    /// The vector scaled by a dimensionless factor.
    fn mul(self, vector: Vec2<U>) -> Vec2<U> {
        vector * self
    }
}

impl<U> Div<f64> for Vec2<U> {
    type Output = Self;

    /// The vector divided by a dimensionless factor.
    fn div(self, divisor: f64) -> Self {
        Self::new(self.x / divisor, self.y / divisor)
    }
}

impl<U> From<Vec2<U>> for [f64; 2] {
    /// The point as `[x, y]`.
    fn from(v: Vec2<U>) -> Self {
        [v.x, v.y]
    }
}

impl<U> From<[f64; 2]> for Vec2<U> {
    /// The pair as a `Vec2` in the unit `U`.
    fn from([x, y]: [f64; 2]) -> Self {
        Self::new(x, y)
    }
}

#[cfg(feature = "cynic")]
impl<U> cynic::QueryFragment for Vec2<U> {
    type SchemaType = schema::Point;
    type VariablesFields = ();
    const TYPE: Option<&'static str> = Some("Point");

    /// `Point { x y }` - the unit `U` is a compile-time tag only and
    /// carries no wire representation.
    fn query(
        builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>,
    ) {
        let mut builder = builder;
        let _ = builder.select_field::<schema::__fields::Point::x, f64>();
        let _ = builder.select_field::<schema::__fields::Point::y, f64>();
    }
}

/// Serde adapter for `#[serde(with)]`: `Vec2<U>` as a compact
/// `[x, y]` pair instead of the default `{"x": .., "y": ..}` map.
///
/// Dataset-style formats like figure profiles or game maps often
/// prefer the tighter pair shape; downstream crates use this module as
/// `#[serde(with = "tangible_objects_model::serde_pair")]`.
pub mod serde_pair {
    use super::Vec2;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;

    /// Serializes the point as `[x, y]`.
    pub fn serialize<U, S: Serializer>(v: &Vec2<U>, serializer: S) -> Result<S::Ok, S::Error> {
        [v.x, v.y].serialize(serializer)
    }

    /// Deserializes `[x, y]` into a `Vec2` in the unit `U`.
    pub fn deserialize<'de, U, D: Deserializer<'de>>(deserializer: D) -> Result<Vec2<U>, D::Error> {
        <[f64; 2]>::deserialize(deserializer).map(Vec2::from)
    }
}

#[cfg(test)]
mod tests {
    use super::Vec2;
    use crate::Millimeters;
    use crate::Pixels;

    #[test]
    fn distance_of_axis_aligned_points() {
        let a = Vec2::<Millimeters>::new(0.0, 0.0);
        let b = Vec2::<Millimeters>::new(3.0, 4.0);
        assert!((a.distance(&b).as_f64() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn centroid_of_triangle() {
        let points = [
            Vec2::<Millimeters>::new(0.0, 0.0),
            Vec2::<Millimeters>::new(6.0, 0.0),
            Vec2::<Millimeters>::new(0.0, 6.0),
        ];
        let c = Vec2::centroid(&points);
        assert!((c.x - 2.0).abs() < 1e-9);
        assert!((c.y - 2.0).abs() < 1e-9);
    }

    #[test]
    fn dot_and_cross_of_perpendiculars() {
        let right = Vec2::<Millimeters>::new(1.0, 0.0);
        let up = Vec2::<Millimeters>::new(0.0, 1.0);
        let dot: f64 = right.dot(&up);
        assert!(dot.abs() < 1e-9);
        let (cross, reverse): (f64, f64) = (right.cross(&up), up.cross(&right));
        assert!((cross - 1.0).abs() < 1e-9);
        assert!((reverse - -1.0).abs() < 1e-9);
        let area: crate::SquareMillimeters = right.cross(&up);
        assert!((area.as_f64() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn dot_and_cross_of_diagonal() {
        let diagonal = Vec2::<Millimeters>::new(1.0, 1.0);
        let axis = Vec2::<Millimeters>::new(1.0, 0.0);
        let (dot, cross): (f64, f64) = (diagonal.dot(&axis), diagonal.cross(&axis));
        assert!((dot - 1.0).abs() < 1e-9);
        assert!((cross - -1.0).abs() < 1e-9);
    }

    #[test]
    fn centroid_of_empty_set_is_origin() {
        let empty: Vec<Vec2<Millimeters>> = Vec::new();
        assert_eq!(Vec2::centroid(&empty), Vec2::new(0.0, 0.0));
    }

    /// Vectors add, subtract, negate and scale component-wise.
    #[test]
    fn arithmetic_ops() {
        let a = Vec2::<Millimeters>::new(10.0, 4.0);
        let b = Vec2::<Millimeters>::new(2.0, 1.0);
        assert_eq!(a + b, Vec2::new(12.0, 5.0));
        assert_eq!(a - b, Vec2::new(8.0, 3.0));
        assert_eq!(-b, Vec2::new(-2.0, -1.0));
        assert_eq!(a * 2.0, Vec2::new(20.0, 8.0));
        assert_eq!(2.0 * b, Vec2::new(4.0, 2.0));
        assert_eq!(a / 2.0, Vec2::new(5.0, 2.0));
    }

    /// `magnitude` measures the vector's length; `perpendicular`
    /// turns it 90° left.
    #[test]
    fn magnitude_and_perpendicular() {
        let vector = Vec2::<Millimeters>::new(3.0, 4.0);
        assert_eq!(vector.magnitude(), Millimeters::new(5.0));
        assert_eq!(vector.perpendicular(), Vec2::new(-4.0, 3.0));
        assert_eq!(vector.normalized(), Vec2::new(0.6, 0.8));
        assert_eq!(Vec2::<Millimeters>::ZERO.normalized(), Vec2::ZERO);
    }

    /// `from_mm`/`from_px` wrap typed components without
    /// `as_f64` at the call site.
    #[test]
    fn from_typed_components() {
        let mm = Vec2::<Millimeters>::from_mm(Millimeters::new(3.0), Millimeters::new(4.0));
        assert_eq!((mm.x, mm.y), (3.0, 4.0));
        let px = Vec2::<Pixels>::from_px(Pixels::new(3.0), Pixels::new(4.0));
        assert_eq!((px.x, px.y), (3.0, 4.0));
    }

    #[test]
    fn array_conversions_roundtrip() {
        let v = Vec2::<Millimeters>::new(1.5, -2.25);
        let arr: [f64; 2] = v.into();
        assert_eq!(arr, [1.5, -2.25]);
        assert_eq!(Vec2::<Millimeters>::from(arr), v);
    }

    #[test]
    fn serde_pair_serializes_as_pair() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Row {
            #[serde(with = "crate::serde_pair")]
            pos: Vec2<Millimeters>,
        }
        let row = Row {
            pos: Vec2::new(80.0, 220.0),
        };
        let json = serde_json::to_string(&row).unwrap_or_default();
        assert_eq!(json, r#"{"pos":[80.0,220.0]}"#);
        assert_eq!(serde_json::from_str::<Row>(&json).ok(), Some(row));
    }
}
