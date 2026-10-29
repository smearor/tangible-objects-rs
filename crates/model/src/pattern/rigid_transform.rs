use crate::Combination;
use crate::Millimeters;
use crate::PointSet;
use crate::RefPoints;
use crate::RotationRad;
use crate::Vec2;

/// Rigid 2D transform (rotation + translation) fitted to the measured
/// points. `mapping[i]` maps `measured[i]` to `refs[mapping[i]]` -
/// positional indexes into the reference slice the transform was
/// fitted on.
pub struct RigidTransform {
    /// Rotation from the reference frame to the measured frame.
    pub angle: RotationRad,
    /// Centroid of the reference points.
    pub ref_center: Vec2<Millimeters>,
    /// Centroid of the measured points.
    pub meas_center: Vec2<Millimeters>,
}

impl RigidTransform {
    /// Applies the transform: `meas_center + R(angle) · (p - ref_center)`.
    pub fn apply(&self, p: Vec2<Millimeters>) -> Vec2<Millimeters> {
        let (sin, cos) = self.angle.sin_cos();
        let dx = p.x - self.ref_center.x;
        let dy = p.y - self.ref_center.y;
        Vec2::new(
            self.meas_center.x + dx * cos - dy * sin,
            self.meas_center.y + dx * sin + dy * cos,
        )
    }

    /// Sum of squared residuals - `Σ d²` over every ref position's
    /// transformed distance to its mapped measured point. The ranking
    /// measure of correspondence enumeration: the lower, the better
    /// the mapping explains the measurement.
    pub fn residual(
        &self,
        refs: &RefPoints,
        measured: PointSet<'_>,
        mapping: Combination<'_>,
    ) -> f64 {
        refs.iter()
            .enumerate()
            .map(|(ri, r)| {
                let measured_index = mapping.iter().position(|&mapped| mapped == ri).unwrap_or(0);
                self.apply(*r).distance(&measured[measured_index]).as_f64()
            })
            .map(|d| d * d)
            .sum()
    }
}
