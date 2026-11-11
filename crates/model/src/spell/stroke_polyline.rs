//! `StrokePolyline` - one finger's gesture path (template or live
//! trajectory), normalized for corridor matching.

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

/// Samples per resampled stroke - enough to keep shape detail,
/// cheap enough to run the Fréchet scan per frame (SPELLS.md).
pub const STROKE_SAMPLES: usize = 64;

/// A stroke as an ordered polyline in millimeters.
///
/// Templates live in *pattern space*: origin at the first point,
/// optionally uniformly scaled to a reference extent. Live
/// trajectories are normalized the same way before matching.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StrokePolyline {
    /// Ordered stroke points in millimeters.
    #[serde(with = "smearor_commons_geometry::serde_points")]
    pub points: Vec<Vec2<Millimeters>>,
}

impl StrokePolyline {
    /// Wraps points as a stroke.
    pub fn new(points: Vec<Vec2<Millimeters>>) -> Self {
        Self { points }
    }

    /// Resamples the stroke to `STROKE_SAMPLES` evenly spaced points
    /// along its arc length (kills sampling-rate noise; the
    /// $-family recognizers' canonical step).
    pub fn resampled(&self) -> Self {
        let count = STROKE_SAMPLES;
        if self.points.len() < 2 {
            return self.clone();
        }
        let total: f64 = self
            .points
            .windows(2)
            .map(|pair| pair[0].distance(&pair[1]).as_f64())
            .sum();
        if total < 1e-6 {
            return self.clone();
        }
        let step = total / (count - 1) as f64;
        let mut out = Vec::with_capacity(count);
        let mut previous = self.points[0];
        out.push(previous);
        let mut walked = 0.0;
        for point in &self.points[1..] {
            let mut seg = previous.distance(point).as_f64();
            while walked + seg >= step && out.len() < count {
                let t = (step - walked) / seg;
                let interpolated = Vec2::<Millimeters>::new(
                    previous.x + (point.x - previous.x) * t,
                    previous.y + (point.y - previous.y) * t,
                );
                out.push(interpolated);
                previous = interpolated;
                walked = 0.0;
                seg = previous.distance(point).as_f64();
            }
            walked += seg;
            previous = *point;
        }
        while out.len() < count {
            out.push(*self.points.last().unwrap_or(&previous));
        }
        Self::new(out)
    }

    /// Translates the stroke so its first point sits at the origin
    /// (position-free matching - a spell is drawable anywhere).
    pub fn translated_to_origin(&self) -> Self {
        let Some(first) = self.points.first() else {
            return self.clone();
        };
        Self::new(
            self.points
                .iter()
                .map(|p| Vec2::<Millimeters>::new(p.x - first.x, p.y - first.y))
                .collect(),
        )
    }

    /// Uniformly scales the stroke so its extent (max distance from
    /// the origin point) becomes `reference` millimeters.
    pub fn scaled_to(&self, reference: Millimeters) -> Self {
        let extent = self
            .points
            .iter()
            .map(|p| p.distance(&Vec2::ZERO).as_f64())
            .fold(0.0_f64, f64::max);
        if extent < 1e-6 {
            return self.clone();
        }
        let factor = reference.as_f64() / extent;
        Self::new(
            self.points
                .iter()
                .map(|p| Vec2::<Millimeters>::new(p.x * factor, p.y * factor))
                .collect(),
        )
    }

    /// The normalization applied to both template and trajectory
    /// before corridor matching: translate to origin, optionally
    /// scale to the template's reference extent.
    pub fn normalized(&self, reference: Millimeters, scale: bool) -> Self {
        let translated = self.translated_to_origin();
        if scale {
            translated.scaled_to(reference)
        } else {
            translated
        }
    }

    /// Discrete Fréchet distance to `other` - the maximum leash the
    /// worst point-pair needs when both paths are walked with
    /// monotonic progress (SPELLS.md's corridor primitive: tolerates
    /// speed differences while enforcing direction and shape).
    pub fn frechet(&self, other: &Self) -> Millimeters {
        let n = self.points.len();
        let m = other.points.len();
        if n == 0 || m == 0 {
            return Millimeters::new(f64::INFINITY);
        }
        // Rolling rows - O(n*m) time, O(m) space.
        let mut prev = vec![f64::INFINITY; m];
        let mut cur = vec![f64::INFINITY; m];
        for i in 0..n {
            for (j, o) in other.points.iter().enumerate() {
                let d = self.points[i].distance(o).as_f64();
                cur[j] = if i == 0 && j == 0 {
                    d
                } else {
                    let best = (if i > 0 { prev[j] } else { f64::INFINITY })
                        .min(if j > 0 { prev[j - 1] } else { f64::INFINITY })
                        .min(if j > 0 { cur[j - 1] } else { f64::INFINITY });
                    d.max(best)
                };
            }
            std::mem::swap(&mut prev, &mut cur);
        }
        Millimeters::new(prev[m - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::StrokePolyline;
    use smearor_commons_geometry::Vec2;

    fn stroke(coords: &[(f64, f64)]) -> StrokePolyline {
        StrokePolyline::new(coords.iter().map(|&(x, y)| Vec2::new(x, y)).collect())
    }

    #[test]
    fn identical_strokes_have_zero_frechet() {
        let a = stroke(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]);
        let resampled_a = a.resampled();
        assert!(resampled_a.frechet(&resampled_a).as_f64() < 1e-9);
    }

    #[test]
    fn same_shape_elsewhere_matches_after_translation() {
        let template = stroke(&[(0.0, 0.0), (10.0, 0.0)])
            .resampled()
            .translated_to_origin();
        let shifted = stroke(&[(100.0, 50.0), (110.0, 50.0)])
            .resampled()
            .translated_to_origin();
        assert!(template.frechet(&shifted).as_f64() < 0.5);
    }

    #[test]
    fn different_shape_fails_corridor() {
        let a = stroke(&[(0.0, 0.0), (50.0, 0.0)]).resampled();
        let b = stroke(&[(0.0, 0.0), (0.0, 50.0)]).resampled();
        assert!(a.frechet(&b).as_f64() > 20.0);
    }

    #[test]
    fn scaled_copy_matches_with_scale_normalization() {
        let a = stroke(&[(0.0, 0.0), (10.0, 5.0), (20.0, 0.0)]).resampled();
        let big = stroke(&[(0.0, 0.0), (20.0, 10.0), (40.0, 0.0)]).resampled();
        let an = a.normalized(20.0.into(), true);
        let bn = big.normalized(20.0.into(), true);
        assert!(an.frechet(&bn).as_f64() < 1.0);
    }

    #[test]
    fn direction_matters() {
        let a = stroke(&[(0.0, 0.0), (50.0, 0.0)]).resampled();
        let b = stroke(&[(50.0, 0.0), (0.0, 0.0)]).resampled();
        // Reversed direction: Frechet enforces monotonic progress -
        // the distance is the stroke length, not zero.
        assert!(a.frechet(&b).as_f64() > 40.0);
    }
}
