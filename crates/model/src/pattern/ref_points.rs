use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use super::Correspondences;
use super::DistanceMultiset;
use super::RotationFit;
use crate::BestScore;
use crate::Combination;
use crate::CombinationBuf;
use crate::Millimeters;
use crate::PatternTolerance;
use crate::PointSet;
use crate::RefIndices;
use crate::RigidTransform;
use crate::RotationRad;
use crate::Vec2;

/// Maximum number of correspondence mappings enumerated for a solve -
/// symmetric footprints admit factorially many; the deterministic
/// first-N pick bounds the cost without changing the residual ranking
/// semantics (ROTATION.md degenerate-input cap).
pub const MAX_CORRESPONDENCES: usize = 64;

/// Positions of a pattern's reference pins - the pin *geometry*
/// (ROTATION.md). The counterpart of [`crate::RefIndices`]: a
/// `RefIndex` names a pin, `RefPoints` carries the positions the
/// indices point into.
///
/// Used for the full `TangiblePattern.ref_points` canonical set and
/// for kept-subset projections (`RefIndices::points_of`) inside
/// correspondence solving.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RefPoints(pub Vec<Vec2<Millimeters>>);

impl RefPoints {
    /// Number of pins in the geometry.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the geometry is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Enumerates every correspondence `measured[i] → self[mapping[i]]`
    /// consistent with per-point distance multisets (within
    /// `tolerance`). Symmetric figures admit several mappings - the
    /// caller picks the one whose fitted transform has the smallest
    /// residual. Mapping indexes are **positional** into this slice;
    /// capped at [`MAX_CORRESPONDENCES`] in deterministic order.
    pub fn correspondences(
        &self,
        measured: PointSet<'_>,
        tolerance: PatternTolerance,
    ) -> Correspondences {
        let tol = tolerance.as_millimeters();
        let ref_sets: Vec<DistanceMultiset> = (0..self.len())
            .map(|i| DistanceMultiset::of(&self.0, i))
            .collect();
        let meas_sets: Vec<DistanceMultiset> = (0..measured.len())
            .map(|i| DistanceMultiset::of(measured.as_slice(), i))
            .collect();
        if self.len() != measured.len() {
            return Correspondences::default();
        }

        // Candidates per measured point: ref positions with an equal
        // distance multiset.
        let candidates: Vec<Vec<usize>> = meas_sets
            .iter()
            .map(|ms| {
                ref_sets
                    .iter()
                    .enumerate()
                    .filter(|(_, rs)| ms.matches_within(rs, tol))
                    .map(|(ri, _)| ri)
                    .collect()
            })
            .collect();

        let mut result = Vec::new();
        let mut mapping = vec![usize::MAX; measured.len()];
        let mut used = vec![false; self.len()];
        enumerate_correspondences(&candidates, 0, &mut mapping, &mut used, &mut result);
        result.into()
    }

    /// Least-squares rigid transform: rotation angle via the summed
    /// cross/dot products of centered points. `mapping[i]` maps
    /// `measured[i]` to `self[mapping[i]]`.
    pub fn similarity_transform(
        &self,
        measured: PointSet<'_>,
        mapping: Combination<'_>,
    ) -> Option<RigidTransform> {
        let ref_center = Vec2::centroid(self);
        let meas_center = Vec2::centroid(measured.as_slice());
        let mut cross = 0.0;
        let mut dot = 0.0;
        for (mi, m) in measured.iter().enumerate() {
            let r = self[mapping[mi]];
            let dr = (r.x - ref_center.x, r.y - ref_center.y);
            let dm = (m.x - meas_center.x, m.y - meas_center.y);
            cross += dr.0 * dm.1 - dr.1 * dm.0;
            dot += dr.0 * dm.0 + dr.1 * dm.1;
        }
        if dot.abs() + cross.abs() < 1e-9 {
            return None;
        }
        Some(RigidTransform {
            angle: RotationRad::new(cross.atan2(dot)),
            ref_center,
            meas_center,
        })
    }

    /// Fits the correspondence of `measured` to `self[kept]` and
    /// returns the lowest-residual `(angle_rad, correspondence)` -
    /// `correspondence[j]` is the **full** reference pin index of
    /// `measured[j]` (ROTATION.md). `None` on degenerate input.
    pub fn fit_rotation(
        &self,
        kept: &RefIndices,
        measured: PointSet<'_>,
        tolerance: PatternTolerance,
    ) -> Option<RotationFit> {
        let kept_points = kept.points_of(self);
        let mut best = BestScore::default();
        for mapping in kept_points.correspondences(measured, tolerance) {
            let Some(transform) =
                kept_points.similarity_transform(measured, mapping.as_combination())
            else {
                continue;
            };
            best.consider(
                (transform.angle, mapping.clone()),
                transform.residual(&kept_points, measured, mapping.as_combination()),
            );
        }
        best.into_value().map(|(angle, mapping)| {
            let full: RefIndices = mapping.iter().map(|&m| kept[m]).collect();
            RotationFit {
                angle,
                correspondence: full,
            }
        })
    }
}

/// Permutation backtracking of `correspondences`: `mapping[index]`
/// takes each unused ref candidate of `candidates[index]`, recursing
/// until every measured position is assigned. Capped at
/// [`MAX_CORRESPONDENCES`] in deterministic order.
fn enumerate_correspondences(
    candidates: &[Vec<usize>],
    index: usize,
    mapping: &mut Vec<usize>,
    used: &mut Vec<bool>,
    result: &mut Vec<CombinationBuf>,
) {
    if result.len() >= MAX_CORRESPONDENCES {
        return;
    }
    if index == candidates.len() {
        result.push(mapping.clone().into());
        return;
    }
    for &ri in &candidates[index] {
        if used[ri] {
            continue;
        }
        used[ri] = true;
        mapping[index] = ri;
        enumerate_correspondences(candidates, index + 1, mapping, used, result);
        used[ri] = false;
    }
}

impl From<Vec<Vec2<Millimeters>>> for RefPoints {
    /// A position vector IS a ref-points geometry.
    fn from(points: Vec<Vec2<Millimeters>>) -> Self {
        Self(points)
    }
}

impl Deref for RefPoints {
    type Target = [Vec2<Millimeters>];

    /// Ref geometries deref to their position slice - positional
    /// indexing (`refs[i]`) and slice methods stay available.
    fn deref(&self) -> &[Vec2<Millimeters>] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::RefPoints;
    use crate::Millimeters;
    use crate::PatternTolerance;
    use crate::RefIndex;
    use crate::RefIndices;
    use crate::Vec2;

    /// Rotates/translates every ref point by `angle` + `translate`.
    fn moved(refs: &RefPoints, angle: f64) -> Vec<Vec2<Millimeters>> {
        let (sin, cos) = angle.sin_cos();
        refs.iter()
            .map(|p| Vec2::new(200.0 + p.x * cos - p.y * sin, 300.0 + p.x * sin + p.y * cos))
            .collect()
    }

    /// An irregular quad solves the planted rotation under jittered
    /// measurements (ROTATION.md testing section).
    #[test]
    fn fit_rotation_recovers_planted_angle() {
        let refs: RefPoints = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(60.0, 5.0),
            Vec2::new(70.0, 50.0),
            Vec2::new(10.0, 45.0),
        ]
        .into();
        let angle = 0.5;
        let jittered: Vec<Vec2<Millimeters>> = moved(&refs, angle)
            .iter()
            .enumerate()
            .map(|(i, p)| Vec2::new(p.x + (i as f64 - 1.5) * 0.4, p.y - i as f64 * 0.3))
            .collect();
        let kept: RefIndices = (0..4).map(RefIndex::new).collect();
        let fit = refs.fit_rotation(&kept, (&jittered).into(), PatternTolerance::from(5.0));
        assert!(
            fit.as_ref()
                .is_some_and(|fit| (fit.angle.as_f64() - angle).abs() < 0.05),
            "fit_rotation must solve an irregular quad within 0.05 rad of {angle}"
        );
        assert_eq!(fit.map(|fit| fit.correspondence.len()), Some(4));
    }

    /// A square admits several equal-residual mappings - the solve is
    /// deterministic (same call twice, same pick), stable but
    /// arbitrary by design.
    #[test]
    fn fit_rotation_square_is_deterministic() {
        let refs: RefPoints = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 40.0),
        ]
        .into();
        let measured = moved(&refs, 0.3);
        let kept: RefIndices = (0..4).map(RefIndex::new).collect();
        let first = refs.fit_rotation(&kept, (&measured).into(), PatternTolerance::from(2.0));
        let second = refs.fit_rotation(&kept, (&measured).into(), PatternTolerance::from(2.0));
        assert_eq!(
            first.as_ref().map(|fit| fit.angle),
            second.as_ref().map(|fit| fit.angle)
        );
        assert!(first.is_some());
    }
}
