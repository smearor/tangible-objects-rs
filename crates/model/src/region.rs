#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;
use crate::Vec2;

/// A rectangular detection region in millimeters.
///
/// Regions restrict where a figure pattern is allowed to match:
/// all touch points of a candidate must lie inside the union of the
/// pattern's regions (PROFILE_REGIONS.md). The bounds are inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "Region",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct Region {
    /// Left bound of the region, in millimeters.
    pub x_min: Millimeters,
    /// Top bound of the region, in millimeters.
    pub y_min: Millimeters,
    /// Right bound of the region, in millimeters.
    pub x_max: Millimeters,
    /// Bottom bound of the region, in millimeters.
    pub y_max: Millimeters,
}

impl Region {
    /// Whether `point` lies inside this region (bounds inclusive).
    pub fn contains(&self, point: &Vec2<Millimeters>) -> bool {
        point.x >= self.x_min.as_f64()
            && point.x <= self.x_max.as_f64()
            && point.y >= self.y_min.as_f64()
            && point.y <= self.y_max.as_f64()
    }

    /// Lower-left corner (`x_min`, `y_min`).
    pub fn min(&self) -> Vec2<Millimeters> {
        Vec2::new(self.x_min.as_f64(), self.y_min.as_f64())
    }

    /// Upper-right corner (`x_max`, `y_max`).
    pub fn max(&self) -> Vec2<Millimeters> {
        Vec2::new(self.x_max.as_f64(), self.y_max.as_f64())
    }

    /// Width of the region (`x_max - x_min`).
    pub fn width(&self) -> Millimeters {
        self.x_max - self.x_min
    }

    /// Height of the region (`y_max - y_min`).
    pub fn height(&self) -> Millimeters {
        self.y_max - self.y_min
    }

    /// Center point of the region.
    pub fn center(&self) -> Vec2<Millimeters> {
        Vec2::new(
            (self.x_min.as_f64() + self.x_max.as_f64()) / 2.0,
            (self.y_min.as_f64() + self.y_max.as_f64()) / 2.0,
        )
    }

    /// Whether the region is degenerate: inverted, zero-sized or
    /// non-finite. A degenerate region contains no points.
    pub fn is_degenerate(&self) -> bool {
        let finite = [self.x_min, self.y_min, self.x_max, self.y_max]
            .iter()
            .all(|v| v.as_f64().is_finite());
        !finite || self.width() <= Millimeters::default() || self.height() <= Millimeters::default()
    }

    /// Whether this region overlaps `other` - two rectangles overlap
    /// when neither lies completely beside or above the other.
    pub fn overlaps(&self, other: &Region) -> bool {
        self.x_min < other.x_max
            && self.x_max > other.x_min
            && self.y_min < other.y_max
            && self.y_max > other.y_min
    }

    /// The region shrunk by `margin` on every side - negative or
    /// degenerate results are clamped to an empty region.
    pub fn inset(&self, margin: Millimeters) -> Region {
        Region {
            x_min: self.x_min + margin,
            y_min: self.y_min + margin,
            x_max: self.x_max - margin,
            y_max: self.y_max - margin,
        }
    }
}

/// Serde adapter for `#[serde(with)]`: `Region` as a compact
/// `[x_min, y_min, x_max, y_max]` array instead of the default map.
///
/// Parallel to [`crate::serde_pair`] - dataset-style formats that
/// prefer the tighter array shape use
/// `#[serde(with = "tangible_objects_model::serde_quad")]`.
pub mod serde_quad {
    use super::Region;
    use crate::Millimeters;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;

    /// Serializes the region as `[x_min, y_min, x_max, y_max]`.
    pub fn serialize<S: Serializer>(region: &Region, serializer: S) -> Result<S::Ok, S::Error> {
        [
            region.x_min.as_f64(),
            region.y_min.as_f64(),
            region.x_max.as_f64(),
            region.y_max.as_f64(),
        ]
        .serialize(serializer)
    }

    /// Deserializes `[x_min, y_min, x_max, y_max]` into a `Region`.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Region, D::Error> {
        let [x_min, y_min, x_max, y_max] = <[f64; 4]>::deserialize(deserializer)?;
        Ok(Region {
            x_min: Millimeters::new(x_min),
            y_min: Millimeters::new(y_min),
            x_max: Millimeters::new(x_max),
            y_max: Millimeters::new(y_max),
        })
    }
}

/// `FromStr` accepts `x_min,y_min,x_max,y_max` in millimeters
/// (`--zone 0,0,200,150`).
impl std::str::FromStr for Region {
    type Err = ParseRegionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = text.split(',').map(str::trim).collect();
        let [x_min, y_min, x_max, y_max]: [&str; 4] = parts
            .try_into()
            .map_err(|_| ParseRegionError(text.to_string()))?;
        let parse = |part: &str| {
            part.parse::<f64>()
                .map(Millimeters::new)
                .map_err(|_| ParseRegionError(text.to_string()))
        };
        Ok(Region {
            x_min: parse(x_min)?,
            y_min: parse(y_min)?,
            x_max: parse(x_max)?,
            y_max: parse(y_max)?,
        })
    }
}

/// A `--zone` argument that is not four comma-separated millimeter values.
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid region '{0}' - expected x_min,y_min,x_max,y_max in mm")]
pub struct ParseRegionError(String);

#[cfg(test)]
mod tests {
    use super::Region;
    use crate::Millimeters;
    use crate::Vec2;

    fn half(x_min: f64, x_max: f64) -> Region {
        Region {
            x_min: Millimeters::new(x_min),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(x_max),
            y_max: Millimeters::new(1000.0),
        }
    }

    #[test]
    fn contains_inside_and_on_bounds() {
        let region = half(0.0, 500.0);
        assert!(region.contains(&Vec2::new(0.0, 10.0)));
        assert!(region.contains(&Vec2::new(500.0, 10.0)));
        assert!(region.contains(&Vec2::new(250.0, 900.0)));
    }

    #[test]
    fn rejects_outside() {
        let region = half(0.0, 500.0);
        assert!(!region.contains(&Vec2::new(501.0, 10.0)));
        assert!(!region.contains(&Vec2::new(-1.0, 10.0)));
    }

    #[test]
    fn accessors() {
        let region = Region {
            x_min: Millimeters::new(10.0),
            y_min: Millimeters::new(20.0),
            x_max: Millimeters::new(110.0),
            y_max: Millimeters::new(70.0),
        };
        assert_eq!(region.min(), Vec2::new(10.0, 20.0));
        assert_eq!(region.max(), Vec2::new(110.0, 70.0));
        assert_eq!(region.width().as_f64(), 100.0);
        assert_eq!(region.height().as_f64(), 50.0);
        assert_eq!(region.center(), Vec2::new(60.0, 45.0));
        assert!(!region.is_degenerate());
    }

    #[test]
    fn detects_degenerate_regions() {
        for (x_min, y_min, x_max, y_max) in [
            (0.0, 0.0, 0.0, 10.0),
            (0.0, 0.0, 10.0, 0.0),
            (10.0, 0.0, 5.0, 10.0),
            (0.0, 0.0, f64::NAN, 1.0),
        ] {
            let region = Region {
                x_min: Millimeters::new(x_min),
                y_min: Millimeters::new(y_min),
                x_max: Millimeters::new(x_max),
                y_max: Millimeters::new(y_max),
            };
            assert!(region.is_degenerate(), "{region:?}");
        }
    }

    #[test]
    fn serde_quad_serializes_as_array() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Row {
            #[serde(with = "crate::serde_quad")]
            region: Region,
        }
        let row = Row {
            region: Region {
                x_min: Millimeters::new(0.0),
                y_min: Millimeters::new(0.0),
                x_max: Millimeters::new(600.0),
                y_max: Millimeters::new(420.0),
            },
        };
        let json = serde_json::to_string(&row).unwrap_or_default();
        assert_eq!(json, r#"{"region":[0.0,0.0,600.0,420.0]}"#);
        assert_eq!(serde_json::from_str::<Row>(&json).ok(), Some(row));
    }
}
