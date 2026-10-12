use async_graphql::SimpleObject;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A 2D position in millimeters.
#[derive(Debug, Clone, SimpleObject)]
pub struct Point {
    /// Horizontal position in millimeters.
    pub x: f64,
    /// Vertical position in millimeters.
    pub y: f64,
}

impl From<Vec2<Millimeters>> for Point {
    fn from(pos: Vec2<Millimeters>) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}
