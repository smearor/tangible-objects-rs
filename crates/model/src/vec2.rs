use std::marker::PhantomData;

/// A 2D point or vector tagged with its coordinate unit `U`.
///
/// The phantom unit distinguishes positions in millimeters
/// (`Vec2<Millimeters>`), raw device units (`Vec2<RawAxisValue>`) and screen
/// pixels (`Vec2<Pixels>`) so they cannot be mixed up. Components stay plain
/// `f64`; the unit only labels which coordinate space the point lives in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2<U> {
    /// X coordinate in the unit `U`.
    pub x: f64,
    /// Y coordinate in the unit `U`.
    pub y: f64,
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
    pub fn distance(&self, other: &Vec2<U>) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
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

#[cfg(test)]
mod tests {
    use super::Vec2;
    use crate::Millimeters;

    #[test]
    fn distance_of_axis_aligned_points() {
        let a = Vec2::<Millimeters>::new(0.0, 0.0);
        let b = Vec2::<Millimeters>::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-9);
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
}
