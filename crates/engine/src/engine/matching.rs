use std::collections::HashSet;

use tangible_objects_model::CandidateReport;
use tangible_objects_model::ClaimPool;
use tangible_objects_model::Combination;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::MatchScore;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::PatternSet;
use tangible_objects_model::PointSet;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchSet;
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
pub(crate) fn find_matches(touches: TouchSet<'_>, patterns: PatternSet<'_>) -> Vec<PatternMatch> {
    find_matches_inner(
        touches,
        patterns,
        &mut NoDiagnostics,
        &HashSet::new(),
        &CompletionConfig::default(),
    )
}

/// Like [`find_matches`], excluding `exclude` touch indices from every
/// candidate - the reserved slots of incumbent slot bindings.
pub(crate) fn find_matches_excluding(
    touches: TouchSet<'_>,
    patterns: PatternSet<'_>,
    exclude: &HashSet<usize>,
    completion: &CompletionConfig,
) -> Vec<PatternMatch> {
    find_matches_inner(touches, patterns, &mut NoDiagnostics, exclude, completion)
}

/// Like [`find_matches`], additionally reporting every tested point
/// combination - matched or rejected - to `diagnostics`.
///
/// Used by the recording pipeline to capture deviation data for offline
/// tolerance analysis.
pub(crate) fn find_matches_with_diagnostics(
    touches: TouchSet<'_>,
    patterns: PatternSet<'_>,
    diagnostics: &mut impl CandidateDiagnostics,
    exclude: &HashSet<usize>,
    completion: &CompletionConfig,
) -> Vec<PatternMatch> {
    find_matches_inner(touches, patterns, diagnostics, exclude, completion)
}

/// Evaluates a single point combination against one pattern (exact only).
///
/// Used for incumbent re-matching in slot-affinity tracking
/// (TRACKING_STABILIZATION.md): a tracked tangible re-matches on the
/// slots it owned in the previous frame before generic matching runs.
/// `CompletionConfig::affinity_factor` widens the acceptance threshold
/// (`match_score <= factor`) - the slot binding already proves identity,
/// so motion distortion must not lose the match.
pub(crate) fn try_pattern(
    pattern: &TangiblePattern,
    combo: Combination<'_>,
    touches: TouchSet<'_>,
    completion: &CompletionConfig,
) -> Option<PatternMatch> {
    let points = touches.positions_of(combo.as_slice());
    (pattern.match_score((&points).into()) <= completion.affinity_factor.as_f64()
        && pattern.accepts((&points).into()))
    .then(|| build_match(pattern, points.as_point_set(), combo, false, Vec::new()))
}

/// Like [`try_pattern`] for a partial (`num_points - 1`) combination;
/// also claims the predicted missing pin(s) from `pool` when completion
/// is enabled.
pub(crate) fn try_partial(
    pattern: &TangiblePattern,
    combo: Combination<'_>,
    touches: TouchSet<'_>,
    completion: &CompletionConfig,
    pool: ClaimPool<'_>,
) -> Option<PatternMatch> {
    let points = touches.positions_of(combo.as_slice());
    let missing = pattern.num_points.as_usize().checked_sub(combo.len())?;
    pattern
        .matches_partial_missing_scaled(
            (&points).into(),
            missing,
            completion.affinity_factor.as_f64(),
        )
        .filter(|_| pattern.accepts((&points).into()))
        .map(|matched| {
            let claimed = claim_missing(
                pattern,
                &matched.kept_indices,
                points.as_point_set(),
                combo,
                pool,
                touches,
                completion,
            );
            build_match(pattern, points.as_point_set(), combo, true, claimed)
        })
}

fn find_matches_inner(
    touches: TouchSet<'_>,
    patterns: PatternSet<'_>,
    diagnostics: &mut impl CandidateDiagnostics,
    exclude: &HashSet<usize>,
    completion: &CompletionConfig,
) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    let mut used_indices: HashSet<usize> = HashSet::new();

    // Sort by pattern ID for a deterministic evaluation order.
    let mut sorted_patterns: Vec<&TangiblePattern> = patterns.iter().collect();
    sorted_patterns.sort_by_key(|p| p.id);

    for pattern in sorted_patterns {
        // A missing pin may still match via the partial (num_points - 1)
        // reference - patterns with stored reference points stay eligible
        // one touch short of the full pin count.
        let partial_possible = pattern.ref_points.is_some()
            && pattern.num_points >= 4
            && touches.len() + 1 >= pattern.num_points;
        if touches.len() < pattern.num_points && !partial_possible {
            continue;
        }

        // Indices of touch points not yet assigned to a figure and not
        // reserved by an incumbent slot binding (slot affinity).
        let available: Vec<usize> = (0..touches.len())
            .filter(|i| !used_indices.contains(i) && !exclude.contains(i))
            .collect();

        if available.len() < pattern.num_points
            && !(partial_possible && available.len() + 1 >= pattern.num_points)
        {
            continue;
        }

        let mut best: Option<(PatternMatch, f64)> = None;

        for combo in combinations(&available, pattern.num_points.as_usize()) {
            let points = touches.positions_of(&combo);

            let matched = pattern.matches((&points).into()) && pattern.accepts((&points).into());
            diagnostics
                .report(|| exact_report(pattern, points.as_point_set(), (&combo).into(), matched));
            if matched {
                // Score-based ranking (TRAINING.md 3.2): the normalized
                // max-deviation score - equivalent to total error under a
                // scalar tolerance, decisive under per-index tolerances.
                let score = pattern.match_score((&points).into());
                let candidate = build_match(
                    pattern,
                    points.as_point_set(),
                    (&combo).into(),
                    false,
                    Vec::new(),
                );
                consider(&mut best, candidate, score);
            }
        }

        // Partial match: with reference points, try N-k points when a
        // figure loses contacts (e.g. a tilting object). `max_missing`
        // bounds how many pins may be missing (1 or 2); 4+ pin figures
        // only. A successful partial may then *claim* the predicted
        // missing pins from still-unassigned touches (completion).
        //
        // Spawn gate: partials may only *create* a tangible for
        // `spawn_on_partial` patterns - reduced contact sets of other
        // figures must not spawn phantoms (SPAWN_AS_PARTIAL.md).
        if best.is_none()
            && pattern.spawn_on_partial
            && pattern.ref_points.is_some()
            && available.len() + completion.max_missing as usize >= pattern.num_points
            && pattern.num_points >= 4
        {
            let max_missing = (completion.max_missing as usize).min(pattern.num_points - 1);
            for missing in 1..=max_missing {
                let partial_n = pattern.num_points - missing;
                for combo in combinations(&available, partial_n) {
                    let points = touches.positions_of(&combo);

                    let result = pattern
                        .matches_partial_missing((&points).into(), missing)
                        .filter(|_| pattern.accepts((&points).into()));
                    diagnostics.report(|| {
                        partial_report(
                            pattern,
                            points.as_point_set(),
                            (&combo).into(),
                            result.is_some(),
                        )
                    });
                    if let Some(matched) = result {
                        // Partial candidates share the same (scalar × 1.5)
                        // scale - ranking by error is ranking by
                        // normalized score here.
                        let score =
                            matched.match_error.as_f64() / (pattern.tolerance.as_f64() * 1.5);
                        let free: Vec<usize> = available
                            .iter()
                            .copied()
                            .filter(|index| !combo.contains(index))
                            .collect();
                        let claimed = claim_missing(
                            pattern,
                            &matched.kept_indices,
                            points.as_point_set(),
                            (&combo).into(),
                            (&free).into(),
                            touches,
                            completion,
                        );
                        let candidate = build_match(
                            pattern,
                            points.as_point_set(),
                            (&combo).into(),
                            true,
                            claimed,
                        );
                        consider(&mut best, candidate, score);
                    }
                }
            }
        }

        if let Some((pattern_match, _)) = best {
            // Matched pins and completion-claimed pins are both consumed -
            // a claimed point belongs to this figure this frame even when
            // `persist` is off (persistence only governs future binding).
            for index in pattern_match.all_indices() {
                used_indices.insert(index);
            }
            matches.push(pattern_match);
        }
    }

    matches
}

/// Claims the predicted positions of a partial match's missing pins.
///
/// `kept_refs` are the reference indices that matched; `points`/`combo`
/// the measured points. For every missing reference pin the expected
/// position is predicted via [`TangiblePattern::predict_missing`] and the
/// nearest free touch inside `radius_factor` × tolerance is claimed.
fn claim_missing(
    pattern: &TangiblePattern,
    kept_refs: &[usize],
    points: PointSet<'_>,
    combo: Combination<'_>,
    pool: ClaimPool<'_>,
    touches: TouchSet<'_>,
    completion: &CompletionConfig,
) -> Vec<usize> {
    let radius = pattern.tolerance.as_f64() * completion.radius_factor;
    let mut claimed = Vec::new();
    let mut taken: HashSet<usize> = combo.iter().copied().collect();
    for missing in (0..pattern.num_points.as_usize()).filter(|index| !kept_refs.contains(index)) {
        let Some(predicted) = pattern.predict_missing(kept_refs, points, missing) else {
            continue;
        };
        let nearest = pool
            .iter()
            .filter(|index| !taken.contains(index))
            .min_by(|&&a, &&b| {
                touches[a]
                    .pos
                    .distance(&predicted)
                    .total_cmp(&touches[b].pos.distance(&predicted))
            });
        if let Some(&index) =
            nearest.filter(|&&index| touches[index].pos.distance(&predicted).as_f64() <= radius)
        {
            claimed.push(index);
            taken.insert(index);
        }
    }
    claimed
}

/// Builds a [`CandidateReport`] for an exact (full-pin) combination.
fn exact_report(
    pattern: &TangiblePattern,
    points: PointSet<'_>,
    combo: Combination<'_>,
    matched: bool,
) -> CandidateReport {
    let measured_distances = points.distances();
    let deviations = measured_distances.deviations(&pattern.sorted_distances);
    CandidateReport {
        pattern_id: pattern.id,
        pattern_name: pattern.name.clone(),
        touch_indices: combo.as_slice().to_vec(),
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
/// subset - the closest geometric explanation, matched or not.
fn partial_report(
    pattern: &TangiblePattern,
    points: PointSet<'_>,
    combo: Combination<'_>,
    matched: bool,
) -> CandidateReport {
    let measured_distances = points.distances();
    let (deviations, match_error) = (0..pattern.num_points.as_usize())
        .filter_map(|skip| pattern.partial_reference(skip))
        .map(|reference| {
            let deviations = TangiblePattern::deviations(&measured_distances, &reference.distances);
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
        touch_indices: combo.as_slice().to_vec(),
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
    points: PointSet<'_>,
    combo: Combination<'_>,
    partial: bool,
    claimed: Vec<usize>,
) -> PatternMatch {
    let center = Vec2::centroid(points.as_slice());
    let name = if partial {
        format!("{} (partial)", pattern.name).into()
    } else {
        pattern.name.clone()
    };
    PatternMatch {
        pattern_id: pattern.id,
        pattern_name: name,
        spawn_stability: pattern.spawn_stability,
        center,
        rotation_rad: PatternMatch::rotation_of(points, &center),
        touch_indices: combo.as_slice().to_vec(),
        partial,
        claimed_indices: claimed,
    }
}

/// Keeps the better of two candidates - lowest normalized score wins
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
    use super::find_matches_excluding;
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
        let patterns = vec![TangiblePattern::from_points(
            1,
            "triangle",
            (&reference).into(),
            5.0,
        )];

        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
            touch_point(3, 999.0, 999.0), // distractor
        ];

        let matches = find_matches((&touches).into(), (&patterns).into());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_name.as_str(), "triangle");
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
        let patterns = vec![TangiblePattern::from_points(
            1,
            "triangle",
            (&reference).into(),
            5.0,
        )];
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 30.0, 0.0),
            touch_point(2, 0.0, 60.0),
        ];
        assert!(find_matches((&touches).into(), (&patterns).into()).is_empty());
    }

    #[test]
    fn diagnostics_report_every_tested_combination() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let patterns = vec![TangiblePattern::from_points(
            1,
            "triangle",
            (&reference).into(),
            5.0,
        )];
        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
            touch_point(3, 999.0, 999.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(
            (&touches).into(),
            (&patterns).into(),
            &mut reports,
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

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
        let patterns = vec![TangiblePattern::from_points(
            1,
            "triangle",
            (&reference).into(),
            5.0,
        )];
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 30.0, 0.0),
            touch_point(2, 0.0, 60.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(
            (&touches).into(),
            (&patterns).into(),
            &mut reports,
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

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
        let mut pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        pattern.spawn_on_partial = true;
        let patterns = vec![pattern];
        // One pin missing - three square contacts plus a distractor touch
        // (partial matching requires at least num_points active touches).
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 40.0, 0.0),
            touch_point(2, 0.0, 40.0),
            touch_point(3, 999.0, 999.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(
            (&touches).into(),
            (&patterns).into(),
            &mut reports,
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].pattern_name.as_str().ends_with("(partial)"));
        // C(4,4) = 1 exact candidate + C(4,3) = 4 partial candidates.
        assert_eq!(reports.len(), 5);
        let partials: Vec<_> = reports.iter().filter(|report| report.partial).collect();
        assert_eq!(partials.len(), 4);
        assert_eq!(partials.iter().filter(|report| report.matched).count(), 1);
        // A 3-pin subset has C(3,2) = 3 distances.
        assert!(partials.iter().all(|report| report.deviations.len() == 3));
    }

    #[test]
    fn region_rejects_a_geometrically_valid_match() {
        use tangible_objects_model::Region;
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&reference).into(), 5.0);
        // Only the left half of the surface may match.
        pattern.regions = vec![Region {
            x_min: tangible_objects_model::Millimeters::new(0.0),
            y_min: tangible_objects_model::Millimeters::new(0.0),
            x_max: tangible_objects_model::Millimeters::new(200.0),
            y_max: tangible_objects_model::Millimeters::new(1000.0),
        }];
        let patterns = vec![pattern];

        let inside = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
        ];
        assert_eq!(find_matches((&inside).into(), (&patterns).into()).len(), 1);

        // Same fingerprint on the right half: region rejects it.
        let outside = vec![
            touch_point(0, 400.0, 100.0),
            touch_point(1, 450.0, 100.0),
            touch_point(2, 400.0, 150.0),
        ];
        assert!(find_matches((&outside).into(), (&patterns).into()).is_empty());
    }

    #[test]
    fn region_boundary_point_rejects_the_candidate() {
        use tangible_objects_model::Region;
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&reference).into(), 5.0);
        pattern.regions = vec![Region {
            x_min: tangible_objects_model::Millimeters::new(0.0),
            y_min: tangible_objects_model::Millimeters::new(0.0),
            x_max: tangible_objects_model::Millimeters::new(200.0),
            y_max: tangible_objects_model::Millimeters::new(1000.0),
        }];
        let patterns = vec![pattern];

        // Two points inside, one point outside - the candidate is
        // rejected as a whole (conservative rule).
        let straddling = vec![
            touch_point(0, 150.0, 100.0),
            touch_point(1, 200.0, 100.0),
            touch_point(2, 150.0, 150.0),
        ];
        // Distances: 50, 50, ~70.7 - geometrically valid, but point 1 sits
        // at x=200 (inclusive boundary: still inside). Use x=210 instead.
        let outside = vec![
            touch_point(0, 160.0, 100.0),
            touch_point(1, 210.0, 100.0),
            touch_point(2, 160.0, 150.0),
        ];
        assert_eq!(
            find_matches((&straddling).into(), (&patterns).into()).len(),
            1
        );
        assert!(find_matches((&outside).into(), (&patterns).into()).is_empty());
    }

    #[test]
    fn partial_match_with_missing_touch() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        pattern.spawn_on_partial = true;
        let patterns = vec![pattern];
        // True missing-pin case: only num_points - 1 touches exist (a pin
        // lost contact entirely - e.g. Figur8 with one pin swallowed).
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 40.0, 0.0),
            touch_point(2, 0.0, 40.0),
        ];

        let mut reports = Vec::new();
        let matches = find_matches_with_diagnostics(
            (&touches).into(),
            (&patterns).into(),
            &mut reports,
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].pattern_name.as_str().ends_with("(partial)"));
        // No exact C(3,4) combos are possible - only the C(3,3) = 1 partial.
        assert_eq!(reports.len(), 1);
        assert!(reports[0].partial);
        assert!(reports[0].matched);
    }

    #[test]
    fn partial_match_claims_predicted_missing_pin() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        pattern.spawn_on_partial = true;
        let patterns = vec![pattern];
        // Three matched pins at translated positions + a fourth touch near
        // the predicted missing pin (translated (40,40) → (140,140)).
        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 140.0, 100.0),
            touch_point(2, 100.0, 140.0),
            // ~8.5mm from predicted (140,140): too far for exact
            // (tolerance 5), inside the claim radius (5 x 2 = 10).
            touch_point(3, 146.0, 146.0),
            touch_point(4, 999.0, 999.0), // out of claim radius
        ];

        let matches = find_matches_excluding(
            (&touches).into(),
            (&patterns).into(),
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].is_partial());
        // The near-predicted touch is claimed, the distant one is not.
        assert_eq!(matches[0].claimed_indices, vec![3]);
        assert!(
            matches[0]
                .all_indices()
                .collect::<Vec<usize>>()
                .contains(&3)
        );
        assert!(
            !matches[0]
                .all_indices()
                .collect::<Vec<usize>>()
                .contains(&4)
        );
    }

    #[test]
    fn partial_match_does_not_claim_distant_touch() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(0.0, 40.0),
            Vec2::new(40.0, 40.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        pattern.spawn_on_partial = true;
        let patterns = vec![pattern];
        // Fourth touch 30mm from the predicted position - outside the
        // claim radius (tolerance 5 × factor 2 = 10mm).
        let touches = vec![
            touch_point(0, 0.0, 0.0),
            touch_point(1, 40.0, 0.0),
            touch_point(2, 0.0, 40.0),
            touch_point(3, 70.0, 40.0),
        ];

        let matches = find_matches_excluding(
            (&touches).into(),
            (&patterns).into(),
            &std::collections::HashSet::new(),
            &tangible_objects_model::CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].is_partial());
        assert!(matches[0].claimed_indices.is_empty());
    }
}
