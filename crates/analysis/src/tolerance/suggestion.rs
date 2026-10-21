use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::PatternTolerances;

/// Suggested tolerances for one pattern (TRAINING.md 2.2).
#[derive(Debug, Serialize, Deserialize)]
pub struct ToleranceSuggestion {
    /// Name of the figure profile.
    pub pattern_name: String,
    /// Tolerance currently configured (from the recorded candidates).
    pub current_tolerance: PatternTolerance,
    /// Scalar recommendation: largest per-index suggestion - the conservative
    /// single value compatible with today's `tolerance` field.
    pub suggested_scalar: PatternTolerance,
    /// Per-distance-index suggestion (`tolerances[i]` parallel to
    /// `sorted_distances[i]`): `p99[i] × safety factor`.
    pub suggested_tolerances: PatternTolerances,
}

impl ToleranceSuggestion {
    /// Renders the suggestion as a TOML assignment for the profile file.
    pub fn toml(&self) -> String {
        let values: Vec<String> = self
            .suggested_tolerances
            .iter()
            .map(|value| format!("{:.2}", value.as_f64()))
            .collect();
        format!("tolerances = [{}]", values.join(", "))
    }
}
