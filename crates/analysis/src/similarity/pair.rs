use serde::Serialize;
use tangible_objects_model::ContactCount;
use tangible_objects_model::Distances;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::PointSet;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::Vec2;

use super::MatchRisk;
use super::subset_collision::SubsetCollision;

/// Collision assessment of one pattern pair.
///
/// `min_max_deviation` is the smallest achievable maximum deviation
/// over all same-size subset fingerprints (`contact_count` pins kept)
/// - the geometric phantom surface. A value ≤ tolerance means a real
///   contact set can satisfy both fingerprints.
#[derive(Debug, Serialize)]
pub struct SimilarityPair {
    /// First profile of the pair.
    pub a: PatternName,
    /// Second profile of the pair.
    pub b: PatternName,
    /// Contact count of the closest subset pairing - `num_points` means
    /// the exact fingerprints collide.
    pub contact_count: ContactCount,
    /// Smallest achievable maximum deviation across subset pairings.
    pub min_max_deviation: Millimeters,
    /// Risk classification derived from `min_max_deviation` and the
    /// pair's tolerances.
    pub risk: MatchRisk,
}

impl SimilarityPair {
    /// Finds the closest fingerprint collision between two patterns over
    /// all same-size subset pairings (k = min(n_a, n_b) down to 3).
    ///
    /// Returns `None` when no comparable fingerprints exist (different
    /// sizes and no `ref_points` to derive subsets).
    pub fn closest(a: &TangiblePattern, b: &TangiblePattern) -> Option<Self> {
        let collision = Self::closest_collision(a, b)?;
        Some(Self {
            a: a.name.clone(),
            b: b.name.clone(),
            contact_count: collision.contact_count,
            min_max_deviation: collision.min_max_deviation,
            risk: Self::risk_of(collision.min_max_deviation, a.tolerance, b.tolerance),
        })
    }

    /// Finds the closest fingerprint collision between two patterns over
    /// all same-size subset pairings.
    ///
    /// `contact_count == num_points` means even the exact fingerprints
    /// nearly collide.
    fn closest_collision(a: &TangiblePattern, b: &TangiblePattern) -> Option<SubsetCollision> {
        if let (Some(points_a), Some(points_b)) = (a.ref_points.as_ref(), b.ref_points.as_ref()) {
            let max_k = a.num_points.min(b.num_points).as_usize();
            let mut best: Option<SubsetCollision> = None;
            for k in (3..=max_k).rev() {
                for keep_a in combinations(a.num_points.as_usize(), k) {
                    let fp_a = subset_fingerprint(a, points_a, &keep_a);
                    for keep_b in combinations(b.num_points.as_usize(), k) {
                        let fp_b = subset_fingerprint(b, points_b, &keep_b);
                        let deviation = fp_a.max_deviation(&fp_b);
                        if best
                            .as_ref()
                            .is_none_or(|c| deviation < c.min_max_deviation)
                        {
                            best = Some(SubsetCollision {
                                contact_count: ContactCount::new(k as u32),
                                min_max_deviation: deviation,
                            });
                        }
                    }
                }
            }
            return best;
        }

        // Fallback without ref_points: exact fingerprint comparison when
        // both patterns carry the same number of distances.
        if a.sorted_distances.len() == b.sorted_distances.len() {
            return Some(SubsetCollision {
                contact_count: ContactCount::new(a.num_points.max(b.num_points).as_u32()),
                min_max_deviation: a.sorted_distances.max_deviation(&b.sorted_distances),
            });
        }
        None
    }

    /// Classifies the risk: `Exact` within the stricter tolerance, else
    /// `Partial` within 1.5x tolerance, else `Safe`.
    fn risk_of(
        deviation: Millimeters,
        tol_a: PatternTolerance,
        tol_b: PatternTolerance,
    ) -> MatchRisk {
        let tolerance = tol_a.as_f64().max(tol_b.as_f64());
        if deviation.as_f64() <= tolerance {
            MatchRisk::Exact
        } else if deviation.as_f64() <= tolerance * 1.5 {
            MatchRisk::Partial
        } else {
            MatchRisk::Safe
        }
    }
}

/// Sorted pairwise distances of the kept reference points - `k ==
/// num_points` falls back to the full sorted fingerprint (the
/// `partial_reference_multi` path needs at least one skipped pin).
fn subset_fingerprint(
    pattern: &TangiblePattern,
    points: &[Vec2<Millimeters>],
    keep: &[usize],
) -> Distances {
    if keep.len() == pattern.num_points {
        return pattern.sorted_distances.clone();
    }
    let subset: Vec<Vec2<Millimeters>> = keep.iter().map(|&i| points[i]).collect();
    PointSet::new(&subset).distances()
}

/// All `k`-element index subsets of `n` points (lexicographic order).
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    if k == 0 || k > n {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut combo: Vec<usize> = (0..k).collect();
    loop {
        result.push(combo.clone());
        let Some(pos) = (0..k).rev().find(|&i| combo[i] < n - k + i) else {
            break;
        };
        combo[pos] += 1;
        for i in pos + 1..k {
            combo[i] = combo[i - 1] + 1;
        }
    }
    result
}
