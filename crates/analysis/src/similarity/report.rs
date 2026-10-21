use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::PatternSet;

use super::SimilarityPair;

/// Pairwise profile-similarity report - the phantom-surface matrix.
///
/// For every pattern pair the report computes the closest possible
/// fingerprint collision over all same-size `ref_points` subsets: if
/// `A`'s flapping contacts can satisfy `B`'s partial fingerprint, a
/// phantom spawn is geometrically possible. Pairs are sorted ascending
/// by `min_max_deviation` - the top entries are the highest collision
/// risks (and the first candidates for `spawn_on_partial = false`).
#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarityReport {
    /// One entry per distinct pattern pair, sorted by collision risk.
    pub pairs: Vec<SimilarityPair>,
}

impl SimilarityReport {
    /// Computes the pairwise similarity of all loaded patterns.
    ///
    /// Patterns without `ref_points` only participate in the exact
    /// comparison - partial fingerprints cannot be derived from sorted
    /// distances alone.
    pub fn analyze(patterns: PatternSet<'_>) -> Self {
        let mut pairs: Vec<SimilarityPair> = patterns
            .iter()
            .enumerate()
            .flat_map(|(i, a)| {
                patterns[i + 1..]
                    .iter()
                    .filter_map(move |b| SimilarityPair::closest(a, b))
            })
            .collect();
        pairs.sort_by(|a, b| {
            a.min_max_deviation
                .as_f64()
                .total_cmp(&b.min_max_deviation.as_f64())
        });
        Self { pairs }
    }
}

impl Display for SimilarityReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Profil-Aehnlichkeit (Kollisionsflaeche) ===")?;
        writeln!(
            f,
            "{:<20} {:<20} {:>4} {:>10} {:>8}",
            "A", "B", "k", "min-dev", "risk"
        )?;
        for pair in &self.pairs {
            writeln!(
                f,
                "{:<20} {:<20} {:>4} {:>9.2}mm {:>8?}",
                pair.a.as_str(),
                pair.b.as_str(),
                pair.contact_count,
                pair.min_max_deviation.as_f64(),
                pair.risk,
            )?;
        }
        writeln!(f)
    }
}
