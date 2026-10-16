use std::collections::HashSet;

use tangible_objects_model::CandidateReport;
use tangible_objects_model::MatchScore;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::Vec2;

use super::diagnostics::CandidateDiagnostics;
use super::diagnostics::NoDiagnostics;

/// Finds all pattern matches in the current set of touch points.
///
/// Patterns are evaluated in ID order for deterministic results. Per pattern
/// only the combination with the lowest total error is kept; touch points
/// already assigned to a figure are excluded from further matching. If no
/// exact match exists and the pattern stores reference points, a partial
/// match with one missing pin is attempted (figures with 4+ pins only).
pub(crate) fn find_matches(
    touches: &[TouchPoint],
    patterns: &[TangiblePattern],
) -> Vec<PatternMatch> {
    find_matches_inner(touches, patterns, &mut NoDiagnostics)
}

/// Like [`find_matches`], additionally reporting every tested point
/// combination — matched or rejected — to `diagnostics`.
///
/// Used by the recording pipeline to capture deviation data for offline
/// tolerance analysis.
pub(crate) fn find_matches_with_diagnostics(
    touches: &[TouchPoint],
    patterns: &[TangiblePattern],
    diagnostics: &mut impl CandidateDiagnostics,
) -> Vec<PatternMatch> {
    find_matches_inner(touches, patterns, diagnostics)
}

fn find_matches_inner(
    touches: &[TouchPoint],
    patterns: &[TangiblePattern],
    diagnostics: &mut impl CandidateDiagnostics,
) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    let mut used_indices: HashSet<usize> = HashSet::new();

    // Sort by pattern ID for a deterministic evaluation order.
    let mut sorted_patterns: Vec<&TangiblePattern> = patterns.iter().collect();
    sorted_patterns.sort_by_key(|p| p.id);

    for pattern in sorted_patterns {
        if touches.len() < pattern.num_points {
            continue;
        }

        // Indices of touch points not yet assigned to a figure.
        let available: Vec<usize> = (0..touches.len())
            .filter(|i| !used_indices.contains(i))
            .collect();

        if available.len() < pattern.num_points {
            continue;
        }

        let mut best: Option<(PatternMatch, f64)> = None;

        for combo in combinations(&available, pattern.num_points) {
            let points: Vec<Vec2<Millimeters>> = combo.iter().map(|&i| touches[i].pos).collect();

            let matched = pattern.matches(&points);
            diagnostics.report(|| exact_report(pattern, &points, &combo, matched));
            if matched {
                // Score-based ranking (TRAINING.md 3.2): the normalized
                // max-deviation score — equivalent to total error under a
                // scalar tolerance, decisive under per-index tolerances.
                let score = pattern.match_score(&points);
                let candidate = build_match(pattern, &points, &combo, false);
                consider(&mut best, candidate, score);
            }
        }

        // Partial match: with reference points, try N-1 points when a figure
        // loses one contact (e.g. a tilting object). Only for 4+ pin figures.
        if best.is_none()
            && pattern.ref_points.is_some()
            && available.len() >= pattern.num_points - 1
            && pattern.num_points >= 4
        {
            let partial_n = pattern.num_points - 1;
            for combo in combinations(&available, partial_n) {
                let points: Vec<Vec2<Millimeters>> =
                    combo.iter().map(|&i| touches[i].pos).collect();

                let result = pattern.matches_partial(&points);
                diagnostics.report(|| partial_report(pattern, &points, &combo, result.is_some()));
                if let Some((_ref_indices, error)) = result {
                    // Partial candidates share the same (scalar × 1.5) scale —
                    // ranking by error is ranking by normalized score here.
                    let score = error.as_f64() / (pattern.tolerance.as_f64() * 1.5);
                    let candidate = build_match(pattern, &points, &combo, true);
                    consider(&mut best, candidate, score);
                }
            }
        }

        if let Some((pattern_match, _)) = best {
            for &index in &pattern_match.touch_indices {
                used_indices.insert(index);
            }
            matches.push(pattern_match);
        }
    }

    matches
}

/// Builds a [`CandidateReport`] for an exact (full-pin) combination.
fn exact_report(
    pattern: &TangiblePattern,
    points: &[Vec2<Millimeters>],
    combo: &[usize],
    matched: bool,
) -> CandidateReport {
    let measured_distances = TangiblePattern::distances_of(points);
    let deviations = TangiblePattern::deviations(&measured_distances, &pattern.sorted_distances);
    CandidateReport {
        pattern_id: pattern.id,
        pattern_name: pattern.name.clone(),
        touch_indices: combo.to_vec(),
        matched,
        partial: false,
        match_error: deviations.iter().copied().sum(),
        score: Some(MatchScore::new(pattern.score_of(&deviations))),
        tolerances: pattern
            .tolerances
            .iter()
            .map(|tol| tol.as_millimeters())
            .collect(),
        measured_distances,
        deviations,
        tolerance: pattern.tolerance,
    }
}

/// Builds a [`CandidateReport`] for a partial (`num_points - 1`) combination.
///
/// Deviations are reported against the best-fitting one-pin-dropped reference
/// subset — the closest geometric explanation, matched or not.
fn partial_report(
    pattern: &TangiblePattern,
    points: &[Vec2<Millimeters>],
    combo: &[usize],
    matched: bool,
) -> CandidateReport {
    let measured_distances = TangiblePattern::distances_of(points);
    let (deviations, match_error) = (0..pattern.num_points)
        .filter_map(|skip| pattern.partial_reference(skip))
        .map(|(_indices, reference)| {
            let deviations = TangiblePattern::deviations(&measured_distances, &reference);
            let error: Millimeters = deviations.iter().copied().sum();
            (deviations, error)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or_else(|| (Vec::new(), Millimeters::new(0.0)));

    let partial_tolerance = pattern.tolerance.as_f64() * 1.5;
    let score = deviations
        .iter()
        .map(|dev| dev.as_f64() / partial_tolerance)
        .reduce(f64::max)
        .unwrap_or(0.0);

    CandidateReport {
        pattern_id: pattern.id,
        pattern_name: pattern.name.clone(),
        touch_indices: combo.to_vec(),
        matched,
        partial: true,
        measured_distances,
        deviations,
        tolerance: pattern.tolerance,
        tolerances: Vec::new(),
        score: Some(MatchScore::new(score)),
        match_error,
    }
}

/// Builds a [`PatternMatch`] for a matched point combination.
fn build_match(
    pattern: &TangiblePattern,
    points: &[Vec2<Millimeters>],
    combo: &[usize],
    partial: bool,
) -> PatternMatch {
    let center = Vec2::centroid(points);
    let name = if partial {
        format!("{} (partial)", pattern.name)
    } else {
        pattern.name.clone()
    };
    PatternMatch {
        pattern_id: pattern.id,
        pattern_name: name,
        center,
        rotation_rad: PatternMatch::rotation_of(points, &center),
        touch_indices: combo.to_vec(),
    }
}

/// Keeps the better of two candidates — lowest normalized score wins
/// (TRAINING.md 3.2 score-based ranking).
fn consider(best: &mut Option<(PatternMatch, f64)>, candidate: PatternMatch, score: f64) {
    match best {
        None => *best = Some((candidate, score)),
        Some((_, best_score)) if score < *best_score => *best = Some((candidate, score)),
        _ => {}
    }
}

/// Generates all k-element index combinations of `items`.
fn combinations(items: &[usize], k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() || items.len() < k {
        return vec![];
    }

    let mut result = Vec::new();
    let head = items[0];
    let tail = &items[1..];

    for mut combo in combinations(tail, k - 1) {
        combo.insert(0, head);
        result.push(combo);
    }
    result.extend(combinations(tail, k));

    result
}

#[cfg(test)]
mod tests {
    use super::combinations;
    use super::find_matches;
    use super::find_matches_with_diagnostics;
    use tangible_objects_model::TangiblePattern;
    use tangible_objects_model::TouchPoint;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    fn touch_point(slot: u32, x: f64, y: f64) -> TouchPoint {
        TouchPoint {
            slot: TouchPointSlot::new(slot),
            tracking_id: TrackingId::new(slot as i32),
            pos: Vec2::new(x, y),
        }
    }

    #[test]
    fn combinations_generate_binomial_count() {
        assert_eq!(combinations(&[0, 1, 2, 3, 4], 3).len(), 10);
        assert_eq!(combinations(&[0, 1, 2], 3).len(), 1);
        assert!(combinations(&[0, 1], 3).is_empty());
    }

    #[test]
    fn finds_triangle_among_touch_points() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let patterns = vec![TangiblePattern::from_points(1, "triangle", &reference, 5.0)];

        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
            touch_point(3, 999.0, 999.0), // distractor
        ];

        let matches = find_matches(&touches, &patterns);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_name, "triangle");
        assert_eq!(matches[0].touch_indices.len(), 3);
        assert!(!matches[0].touch_indices.contains(&3));
    }

    #[test]
    fn no_match_without_fingerprint() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let patterns = vec![TangiblePattern::from_points(1, "triangle", &reference, 5.0)];
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 30.0, 0.0),
            touch_point(2, 0.0, 60.0),
        ];
        assert!(find_matches(&touches, &patterns).is_empty());
    }

    #[test]
    fn diagnostics_report_every_tested_combination() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let patterns = vec![TangiblePattern::from_points(1, "triangle", &reference, 5.0)];
        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
            touch_point(3, 999.0, 999.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(&touches, &patterns, &mut reports);

        assert_eq!(matches.len(), 1);
        // C(4,3) combinations: one matches, three are rejected.
        assert_eq!(reports.len(), 4);
        assert_eq!(reports.iter().filter(|report| report.matched).count(), 1);
        assert!(reports.iter().all(|report| !report.partial));

        let matched = reports.iter().find(|report| report.matched).expect("match");
        assert_eq!(matched.touch_indices, vec![0, 1, 2]);
        assert!(matched.deviations.iter().all(|d| d.as_f64() <= 5.0));
        // Score-based matching: every report carries its normalized score;
        // matched candidates score <= 1.0, rejected ones above.
        assert!(reports.iter().all(|report| report.score.is_some()));
        assert!(matched.score.is_some_and(|s| s.as_f64() <= 1.0));
        assert!(
            reports
                .iter()
                .filter(|report| !report.matched)
                .all(|report| report.score.is_some_and(|s| s.as_f64() > 1.0))
        );
    }

    #[test]
    fn rejected_candidates_carry_deviations() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let patterns = vec![TangiblePattern::from_points(1, "triangle", &reference, 5.0)];
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 30.0, 0.0),
            touch_point(2, 0.0, 60.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(&touches, &patterns, &mut reports);

        assert!(matches.is_empty());
        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert!(!report.matched);
        assert_eq!(report.measured_distances.len(), 3);
        assert_eq!(report.deviations.len(), 3);
        // |30 - 50| + |60 - 50| + |sqrt(30²+60²) - sqrt(50²+50²)|
        assert!(report.match_error.as_f64() > 25.0);
        assert_eq!(report.tolerance.as_f64(), 5.0);
    }

    #[test]
    fn diagnostics_report_partial_candidates() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let patterns = vec![TangiblePattern::from_points(1, "square", &square, 5.0)];
        // One pin missing — three square contacts plus a distractor touch
        // (partial matching requires at least num_points active touches).
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 40.0, 0.0),
            touch_point(2, 0.0, 40.0),
            touch_point(3, 999.0, 999.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(&touches, &patterns, &mut reports);

        assert_eq!(matches.len(), 1);
        assert!(matches[0].pattern_name.ends_with("(partial)"));
        // C(4,4) = 1 exact candidate + C(4,3) = 4 partial candidates.
        assert_eq!(reports.len(), 5);
        let partials: Vec<_> = reports.iter().filter(|report| report.partial).collect();
        assert_eq!(partials.len(), 4);
        assert_eq!(partials.iter().filter(|report| report.matched).count(), 1);
        // A 3-pin subset has C(3,2) = 3 distances.
        assert!(partials.iter().all(|report| report.deviations.len() == 3));
    }
}
