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
}
