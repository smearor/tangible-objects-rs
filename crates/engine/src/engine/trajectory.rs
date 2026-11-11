//! `Trajectory` - one contact's accumulated gesture stroke in
//! surface millimeters (SPELLS.md).

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

/// Ordered stroke points with cumulative path length - the gesture
/// tracker's per-slot record, decimated while collecting.
#[derive(Debug, Clone, Default)]
pub struct Trajectory {
    /// Ordered stroke points, decimated by the tracker's sample
    /// step.
    pub points: Vec<Vec2<Millimeters>>,
    /// Cumulative path length in millimeters.
    pub path_length: f64,
}

impl Trajectory {
    /// Mean position of every point across a completed stroke set -
    /// the matched gesture's surface anchor.
    pub fn centroid(strokes: &[Vec<Vec2<Millimeters>>]) -> Vec2<Millimeters> {
        let mut sum = Vec2::<Millimeters>::ZERO;
        let mut count = 0.0;
        for stroke in strokes {
            for point in stroke {
                sum = sum + *point;
                count += 1.0;
            }
        }
        if count > 0.0 {
            Vec2::new(sum.x / count, sum.y / count)
        } else {
            Vec2::ZERO
        }
    }
}
