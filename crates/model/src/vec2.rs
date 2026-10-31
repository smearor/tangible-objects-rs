use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;

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
    fn centroid_of_empty_set_is_origin() {
        let empty: Vec<Vec2<Millimeters>> = Vec::new();
        assert_eq!(Vec2::centroid(&empty), Vec2::new(0.0, 0.0));
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
