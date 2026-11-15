//! Pin-correspondence machinery: solve which measured touch index maps
//! to which reference pin, then fit a rigid 2D transform (ROTATION.md).
//! Shared by `predict_missing` (completion) and the tracker's persisted
//! slot→pin mapping.

use crate::Millimeters;
use crate::PatternTolerance;
use crate::Vec2;

/// Maximum number of correspondence mappings enumerated for a solve -
/// symmetric footprints admit factorially many; the deterministic
/// first-N pick bounds the cost without changing the residual ranking
/// semantics (ROTATION.md degenerate-input cap).
pub const MAX_CORRESPONDENCES: usize = 64;

/// Enumerates every correspondence `measured[i] → refs[mapping[i]]`
/// consistent with per-point distance multisets (within `tolerance`).
/// Symmetric figures admit several mappings - the caller picks the one
/// whose fitted transform has the smallest residual. Capped at
/// [`MAX_CORRESPONDENCES`] in deterministic order.
pub fn correspondences(
    refs: &[Vec2<Millimeters>],
    measured: &[Vec2<Millimeters>],
    tolerance: PatternTolerance,
) -> Vec<Vec<usize>> {
    let tol = tolerance.as_millimeters();
    let ref_sets: Vec<Vec<Millimeters>> = (0..refs.len())
        .map(|i| distance_multiset(refs, i))
        .collect();
    let meas_sets: Vec<Vec<Millimeters>> = (0..measured.len())
        .map(|i| distance_multiset(measured, i))
        .collect();
    if refs.len() != measured.len() {
        return Vec::new();
    }

    // Candidates per measured point: ref indices with an equal multiset.
    let candidates: Vec<Vec<usize>> = meas_sets
        .iter()
        .map(|ms| {
            ref_sets
                .iter()
                .enumerate()
                .filter(|(_, rs)| {
                    ms.len() == rs.len()
                        && ms
                            .iter()
                            .zip(rs.iter())
                            .all(|(m, r)| (*m - *r).abs() <= tol)
                })
                .map(|(ri, _)| ri)
                .collect()
        })
        .collect();

    let mut result = Vec::new();
    let mut mapping = vec![usize::MAX; measured.len()];
    let mut used = vec![false; refs.len()];
    fn enumerate(
        candidates: &[Vec<usize>],
        index: usize,
        mapping: &mut Vec<usize>,
        used: &mut Vec<bool>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if result.len() >= MAX_CORRESPONDENCES {
            return;
        }
        if index == candidates.len() {
            result.push(mapping.clone());
            return;
        }
        for &ri in &candidates[index] {
            if used[ri] {
                continue;
            }
            used[ri] = true;
            mapping[index] = ri;
            enumerate(candidates, index + 1, mapping, used, result);
            used[ri] = false;
        }
    }
    enumerate(&candidates, 0, &mut mapping, &mut used, &mut result);
    result
}

/// Fits the correspondence of `measured` to `refs[kept_refs]` and
/// returns the lowest-residual `(angle_rad, correspondence)` -
/// `correspondence[j]` is the **full** reference index of
/// `measured[j]` (ROTATION.md). `None` on degenerate input.
pub fn fit_rotation(
    refs: &[Vec2<Millimeters>],
    kept_refs: &[usize],
    measured: &[Vec2<Millimeters>],
    tolerance: PatternTolerance,
) -> Option<(f64, Vec<usize>)> {
    let kept_points: Vec<Vec2<Millimeters>> =
        kept_refs.iter().map(|&i| refs[i]).collect();
    let mut best: Option<(f64, Vec<usize>, f64)> = None;
    for mapping in correspondences(&kept_points, measured, tolerance) {
        let Some(transform) = similarity_transform(&kept_points, measured, &mapping) else {
            continue;
        };
        let residual: f64 = kept_points
            .iter()
            .enumerate()
            .map(|(ri, r)| {
                let measured_index =
                    mapping.iter().position(|&mapped| mapped == ri).unwrap_or(0);
                transform
                    .apply(*r)
                    .distance(&measured[measured_index])
                    .as_f64()
            })
            .map(|d| d * d)
            .sum();
        if best.as_ref().is_none_or(|(.., r)| residual < *r) {
            best = Some((transform.angle, mapping, residual));
        }
    }
    best.map(|(angle, mapping, _)| {
        let full: Vec<usize> = mapping.iter().map(|&m| kept_refs[m]).collect();
        (angle, full)
    })
}

/// Sorted distances of `points[index]` to all other points.
fn distance_multiset(points: &[Vec2<Millimeters>], index: usize) -> Vec<Millimeters> {
    let mut distances: Vec<Millimeters> = points
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != index)
        .map(|(_, p)| points[index].distance(p))
        .collect();
    distances.sort_by(Millimeters::total_cmp);
    distances
}

/// Rigid 2D transform (rotation + translation) fitted to the measured
/// points. `mapping[i]` maps `measured[i]` to `refs[mapping[i]]`.
pub struct RigidTransform {
    /// Rotation from the reference frame to the measured frame.
    pub angle: f64,
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
}

/// Least-squares rigid transform: rotation angle via the summed
/// cross/dot products of centered points.
pub fn similarity_transform(
    refs: &[Vec2<Millimeters>],
    measured: &[Vec2<Millimeters>],
    mapping: &[usize],
) -> Option<RigidTransform> {
    let ref_center = Vec2::centroid(refs);
    let meas_center = Vec2::centroid(measured);
    let mut cross = 0.0;
    let mut dot = 0.0;
    for (mi, m) in measured.iter().enumerate() {
        let r = refs[mapping[mi]];
        let dr = (r.x - ref_center.x, r.y - ref_center.y);
        let dm = (m.x - meas_center.x, m.y - meas_center.y);
        cross += dr.0 * dm.1 - dr.1 * dm.0;
        dot += dr.0 * dm.0 + dr.1 * dm.1;
    }
    if dot.abs() + cross.abs() < 1e-9 {
        return None;
    }
    Some(RigidTransform {
        angle: cross.atan2(dot),
        ref_center,
        meas_center,
    })
}
