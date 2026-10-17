use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;
use crate::Vec2;

/// Window anchor of a [`crate::FieldSet`]: the top-left corner of the
/// application's window in screen millimeters.
///
/// Field coordinates are interpreted window-relative. When `width` and
/// `height` are present, field bounds are normalized `0.0..=1.0` values
/// scaled to the window size; otherwise they are plain millimeters
/// relative to the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowAnchor {
    /// Window's left edge on screen, in millimeters.
    pub x_offset: Millimeters,
    /// Window's top edge on screen, in millimeters.
    pub y_offset: Millimeters,
    /// Window width in millimeters - enables normalized field bounds.
    pub width: Option<Millimeters>,
    /// Window height in millimeters - enables normalized field bounds.
    pub height: Option<Millimeters>,
}

impl WindowAnchor {
    /// Maps a screen-absolute point (mm) into this window's field
    /// coordinate space: window-relative mm, or `0.0..=1.0` when
    /// `width`/`height` are set.
    pub fn to_local(&self, point: &Vec2<Millimeters>) -> Vec2<Millimeters> {
        let local_x = point.x - self.x_offset.as_f64();
        let local_y = point.y - self.y_offset.as_f64();
        match (self.width, self.height) {
            (Some(width), Some(height)) => {
                Vec2::new(local_x / width.as_f64(), local_y / height.as_f64())
            }
            _ => Vec2::new(local_x, local_y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowAnchor;
    use crate::Millimeters;
    use crate::Vec2;

    #[test]
    fn to_local_subtracts_the_offset() {
        let anchor = WindowAnchor {
            x_offset: Millimeters::new(100.0),
            y_offset: Millimeters::new(50.0),
            width: None,
            height: None,
        };
        let local = anchor.to_local(&Vec2::new(150.0, 80.0));
        assert_eq!(local, Vec2::new(50.0, 30.0));
    }

    #[test]
    fn to_local_normalizes_with_size() {
        let anchor = WindowAnchor {
            x_offset: Millimeters::new(100.0),
            y_offset: Millimeters::new(50.0),
            width: Some(Millimeters::new(200.0)),
            height: Some(Millimeters::new(100.0)),
        };
        let local = anchor.to_local(&Vec2::new(150.0, 100.0));
        assert_eq!(local, Vec2::new(0.25, 0.5));
    }
}
