use std::collections::BTreeMap;
use std::fmt::Display;

use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternTolerance;

use super::ToleranceSuggestion;
use crate::Percentiles;
use crate::Recording;

/// Multiplier applied to the p99 deviation for tolerance suggestions.
///
/// Leaves headroom above the observed noise floor without opening the
/// acceptance volume unnecessarily.
pub const DEFAULT_SAFETY_FACTOR: f64 = 1.25;

/// Per-pair tolerance recommendations over a whole recording (TRAINING.md 2.2).
#[derive(Debug)]
pub struct ToleranceReport {
    /// Safety factor applied to p99.
    pub safety_factor: f64,
    /// One suggestion per pattern that produced at least one matched full
    /// candidate; sorted by pattern ID.
    pub suggestions: Vec<ToleranceSuggestion>,
}

impl ToleranceReport {
    /// Computes suggestions with [`DEFAULT_SAFETY_FACTOR`].
    pub fn analyze(recording: &Recording) -> Self {
        Self::analyze_with_safety(recording, DEFAULT_SAFETY_FACTOR)
    }

    /// Computes per-pair tolerance suggestions from labeled frames.
    ///
    /// On every labeled frame the labeled pattern's best full candidate
    /// contributes its deviation vector - matched or not. Sampling only
    /// matched candidates biases p99 toward the configured tolerance
    /// (survivorship: worse misses never enter the statistics and the
    /// suggestion converges to a value that still misses).
    /// `suggested_tolerances[i] = p99(deviations[i]) × safety_factor`,
    /// clamped to at least 0.1 mm so a perfect measurement does not
    /// collapse the tolerance to zero.
    pub fn analyze_with_safety(recording: &Recording, safety_factor: f64) -> Self {
        let mut deviations: BTreeMap<PatternId, Vec<Vec<Millimeters>>> = BTreeMap::new();
        let mut tolerance: BTreeMap<PatternId, PatternTolerance> = BTreeMap::new();
        let mut names: BTreeMap<PatternId, String> = BTreeMap::new();

        for frame in recording.frames() {
            let Some(label) = &frame.label else {
                continue;
            };
            for candidate in frame.best_candidates().values() {
                if candidate.partial || !candidate.matches_label(label) {
                    continue;
                }
                tolerance.insert(candidate.pattern_id, candidate.tolerance);
                names.insert(candidate.pattern_id, candidate.pattern_name.clone());
                let slots = deviations.entry(candidate.pattern_id).or_default();
                for (index, deviation) in candidate.deviations.iter().enumerate() {
                    if slots.len() <= index {
                        slots.resize_with(index + 1, Vec::new);
                    }
                    slots[index].push(*deviation);
                }
            }
        }

        let suggestions = deviations
            .into_iter()
            .filter_map(|(pattern_id, per_index)| {
                let suggested_tolerances: Vec<PatternTolerance> = per_index
                    .iter()
                    .filter_map(|samples| Percentiles::of(samples.iter().copied()))
                    .map(|stats| PatternTolerance::new((stats.p99 * safety_factor).max(0.1)))
                    .collect();
                if suggested_tolerances.is_empty() {
                    return None;
                }
                let suggested_scalar = suggested_tolerances
                    .iter()
                    .copied()
                    .max_by(|a, b| a.as_f64().total_cmp(&b.as_f64()))
                    .unwrap_or_else(|| PatternTolerance::new(0.0));
                Some(ToleranceSuggestion {
                    current_tolerance: tolerance
                        .get(&pattern_id)
                        .copied()
                        .unwrap_or_else(|| PatternTolerance::new(0.0)),
                    pattern_name: names.get(&pattern_id).cloned().unwrap_or_default(),
                    suggested_scalar,
                    suggested_tolerances,
                })
            })
            .collect();

        Self {
            safety_factor,
            suggestions,
        }
    }
}

impl Display for ToleranceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "== Tolerance suggestions == (p99 × {:.2})",
            self.safety_factor
        )?;
        for suggestion in &self.suggestions {
            writeln!(
                f,
                "  {}: current {:.2} → scalar {:.2} | {}",
                suggestion.pattern_name,
                suggestion.current_tolerance.as_f64(),
                suggestion.suggested_scalar.as_f64(),
                suggestion.toml()
            )?;
        }
        Ok(())
    }
}
