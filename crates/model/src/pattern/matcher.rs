//! `PatternSet` matching - the matcher algorithm: every pattern is
//! evaluated against the touch set, best combination per pattern wins,
//! slot-affinity exclusions and candidate diagnostics flow through.
//! Methods live on [`PatternSet`] - the set is the actor evaluating
//! its patterns (ROTATION.md refactor series).

use crate::TakenIndices;

use super::CandidateDiagnostics;
use super::NoDiagnostics;
use super::PatternSet;
use crate::BestScore;
use crate::CandidateReport;
use crate::Combination;
use crate::CombinationBuf;
use crate::Combinations;
use crate::CompletionConfig;
use crate::MatchScore;
use crate::Millimeters;
use crate::PatternMatch;
use crate::PointSet;
use crate::RefIndices;
use crate::TangiblePattern;
use crate::TouchSet;
use crate::Vec2;

impl<'a> PatternSet<'a> {
    /// Finds all pattern matches in the current set of touch points.
    ///
    /// Patterns are evaluated in ID order for deterministic results. Per pattern
    /// only the combination with the lowest total error is kept; touch points
    /// already assigned to a figure are excluded from further matching. If no
    /// exact match exists and the pattern stores reference points, a partial
    /// match with one missing pin is attempted (figures with 4+ pins only).
    /// Finds all pattern matches in the current set of touch points.
    ///
    /// Patterns are evaluated in ID order for deterministic results. Per
    /// pattern only the combination with the lowest total error is kept;
    /// touch points already assigned to a figure are excluded from further
    /// matching. If no exact match exists and the pattern stores reference
    /// points, a partial match with one missing pin is attempted (figures
    /// with 4+ pins only).
    pub fn find_matches(self, touches: TouchSet<'_>) -> Vec<PatternMatch> {
        self.find_matches_inner(
            touches,
            &mut NoDiagnostics,
            &TakenIndices::default(),
            &CompletionConfig::default(),
        )
        .0
    }

    /// Like [`Self::find_matches`], excluding `exclude` touch indices
    /// from every candidate - the reserved slots of incumbent slot
    /// bindings.
    pub fn find_matches_excluding(
        self,
        touches: TouchSet<'_>,
        exclude: &TakenIndices,
        completion: &CompletionConfig,
    ) -> Vec<PatternMatch> {
        self.find_matches_inner(touches, &mut NoDiagnostics, exclude, completion)
            .0
    }

    /// Like [`Self::find_matches`], additionally reporting every
    /// tested point combination - matched or rejected - to
    /// `diagnostics`.
    ///
    /// Used by the recording pipeline to capture deviation data for
    /// offline tolerance analysis.
    pub fn find_matches_with_diagnostics(
        self,
        touches: TouchSet<'_>,
        diagnostics: &mut impl CandidateDiagnostics,
        exclude: &TakenIndices,
        completion: &CompletionConfig,
    ) -> Vec<PatternMatch> {
        self.find_matches_inner(touches, diagnostics, exclude, completion)
            .0
    }

    /// Like [`Self::find_matches_excluding`], additionally returning the
    /// number of candidate combinations actually evaluated - the
    /// `candidates_tested` metric source (SPATIAL_CLUSTERING.md).
    pub fn find_matches_excluding_counted(
        self,
        touches: TouchSet<'_>,
        exclude: &TakenIndices,
        completion: &CompletionConfig,
        tested: &mut usize,
    ) -> Vec<PatternMatch> {
        let (matches, count) =
            self.find_matches_inner(touches, &mut NoDiagnostics, exclude, completion);
        *tested += count;
        matches
    }

    fn find_matches_inner(
        self,
        touches: TouchSet<'_>,
        diagnostics: &mut impl CandidateDiagnostics,
        exclude: &TakenIndices,
        completion: &CompletionConfig,
    ) -> (Vec<PatternMatch>, usize) {
        let mut matches = Vec::new();
        let mut tested = 0usize;
        let mut used_indices = TakenIndices::default();

        // Sort by pattern ID for a deterministic evaluation order.
        let mut sorted_patterns: Vec<&TangiblePattern> = self.iter().collect();
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
            let taken = used_indices.union(exclude);
            let available = touches.available_indices(&taken);
            let pos_of = |i: usize| touches[i].pos;

            if available.len() < pattern.num_points
                && !(partial_possible && available.len() + 1 >= pattern.num_points)
            {
                continue;
            }

            let mut best = BestScore::default();

            for combo in combinations_within(
                &available,
                pattern.num_points.as_usize(),
                &pos_of,
                pattern.footprint(),
            ) {
                tested += 1;
                let points = touches.positions_of(&combo);

                let matched =
                    pattern.matches((&points).into()) && pattern.accepts((&points).into());
                diagnostics.report(|| {
                    exact_report(pattern, points.as_point_set(), (&combo).into(), matched)
                });
                if matched {
                    // Score-based ranking (TRAINING.md 3.2): the normalized
                    // max-deviation score - equivalent to total error under a
                    // scalar tolerance, decisive under per-index tolerances.
                    let score = pattern.match_score((&points).into());
                    let candidate = pattern.build_match(
                        points.as_point_set(),
                        combo.as_combination(),
                        false,
                        Default::default(),
                    );
                    best.consider(candidate, score);
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
                    for combo in
                        combinations_within(&available, partial_n, &pos_of, pattern.footprint())
                    {
                        tested += 1;
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
                            let score = MatchScore::normalized(
                                matched.match_error.as_millimeters(),
                                pattern.partial_tolerance(),
                            );
                            let free = available.without(combo.as_combination());
                            let claimed = pattern.claim_missing(
                                &matched.kept_indices,
                                points.as_point_set(),
                                (&combo).into(),
                                free.as_claim_pool(),
                                touches,
                                completion,
                            );
                            let mut candidate = pattern.build_match(
                                points.as_point_set(),
                                combo.as_combination(),
                                true,
                                claimed,
                            );
                            candidate.resolve_correspondence(
                                pattern,
                                points.as_point_set(),
                                &matched.kept_indices,
                            );
                            best.consider(candidate, score);
                        }
                    }
                }
            }

            if let Some(mut pattern_match) = best.into_value() {
                // Exact winners solve their pin correspondence once
                // (ROTATION.md); partials solved it at build time, where
                // the kept reference subset is still in scope.
                if !pattern_match.partial {
                    let points = touches.positions_of(&pattern_match.touch_indices);
                    let kept_refs = RefIndices::all(pattern.num_points);
                    pattern_match.resolve_correspondence(
                        pattern,
                        points.as_point_set(),
                        &kept_refs,
                    );
                }
                // Matched pins and completion-claimed pins are both consumed -
                // a claimed point belongs to this figure this frame even when
                // `persist` is off (persistence only governs future binding).
                for index in pattern_match.all_indices() {
                    used_indices.insert(index);
                }
                matches.push(pattern_match);
            }
        }

        (matches, tested)
    }
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
        touch_indices: combo.into(),
        matched,
        partial: false,
        match_error: deviations.total(),
        score: Some(pattern.score_of(&deviations)),
        tolerances: pattern.tolerances.clone(),
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
        .filter_map(|skip| pattern.partial_reference(skip.into()))
        .map(|reference| {
            let deviations = TangiblePattern::deviations(&measured_distances, &reference.distances);
            let error = deviations.total();
            (deviations, error)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or_else(|| (Default::default(), Millimeters::new(0.0).into()));

    let score = deviations.max_score(|_| pattern.partial_tolerance());

    CandidateReport {
        pattern_id: pattern.id,
        pattern_name: pattern.name.clone(),
        touch_indices: combo.into(),
        matched,
        partial: true,
        measured_distances,
        deviations,
        tolerance: pattern.tolerance,
        tolerances: Default::default(),
        score: Some(score),
        match_error,
    }
}

/// DFS context for [`combinations_within`] (SPATIAL_CLUSTERING.md):
/// the fixed enumeration inputs - item pool, arity, position lookup
/// and the pattern's span bound - kept out of the recursion's
/// parameter list.
struct SpanSearch<'a> {
    /// Item pool to draw indices from (touch indices of the frame).
    items: &'a [usize],
    /// Combination arity - how many indices each candidate holds.
    arity: usize,
    /// Position lookup: item (touch index) -> surface position in mm.
    pos_of: &'a dyn Fn(usize) -> Vec2<Millimeters>,
    /// Pairwise-diameter bound - extending beyond prunes the branch.
    max_span: Millimeters,
}

impl SpanSearch<'_> {
    /// Depth-first combination walk: `start` is the next item position,
    /// `combo`/`span` the accumulating prefix and its pairwise diameter.
    fn descend(
        &self,
        start: usize,
        combo: &mut Vec<usize>,
        span: &mut Millimeters,
        result: &mut Vec<CombinationBuf>,
    ) {
        if combo.len() == self.arity {
            result.push(combo.clone().into());
            return;
        }
        let needed = self.arity - combo.len();
        for (offset, &item) in self.items[start..].iter().enumerate() {
            if self.items.len() - (start + offset) < needed {
                break;
            }
            let pos = (self.pos_of)(item);
            let new_span = combo
                .iter()
                .map(|&c| pos.distance(&(self.pos_of)(c)))
                .fold(*span, |acc, d| if d > acc { d } else { acc });
            if new_span > self.max_span {
                continue;
            }
            let old_span = *span;
            *span = new_span;
            combo.push(item);
            self.descend(start + offset + 1, combo, span, result);
            combo.pop();
            *span = old_span;
        }
    }
}

/// Span-pruned k-combination enumeration (SPATIAL_CLUSTERING.md):
/// `pos_of` maps each item to its position; a partial combination
/// whose pairwise diameter exceeds `max_span` prunes the whole branch -
/// adding points never shrinks a diameter, so every superposition
/// would fail the fingerprint anyway. Emits the same lexicographic
/// order as [`combinations`], minus provably impossible candidates.
fn combinations_within(
    items: &[usize],
    k: usize,
    pos_of: &impl Fn(usize) -> Vec2<Millimeters>,
    max_span: Millimeters,
) -> Combinations {
    if k == 0 {
        return Combinations::one_empty();
    }
    let mut result = Vec::new();
    SpanSearch {
        items,
        arity: k,
        pos_of,
        max_span,
    }
    .descend(
        0,
        &mut Vec::with_capacity(k),
        &mut Millimeters::default(),
        &mut result,
    );
    result.into()
}

/// Generates all k-element index combinations of `items` - the
/// unpruned baseline kept for tests.
#[cfg(test)]
fn combinations(items: &[usize], k: usize) -> Combinations {
    if k == 0 {
        return Combinations::one_empty();
    }
    if items.is_empty() || items.len() < k {
        return Combinations::default();
    }

    let mut result = Combinations::new();
    let head = items[0];
    let tail = &items[1..];

    for mut combo in combinations(tail, k - 1) {
        combo.0.insert(0, head);
        result.push(combo);
    }
    result.extend(combinations(tail, k));

    result
}

#[cfg(test)]
mod tests {
    use super::combinations;
    use super::combinations_within;
    use crate::CompletionConfig;
    use crate::Millimeters;
    use crate::PatternSet;
    use crate::Region;
    use crate::TakenIndices;
    use crate::TangiblePattern;
    use crate::TouchPoint;
    use crate::TouchPointSlot;
    use crate::TrackingId;
    use crate::Vec2;

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

        let matches = PatternSet::from(&patterns[..]).find_matches((&touches).into());
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
        assert!(
            PatternSet::from(&patterns[..])
                .find_matches((&touches).into())
                .is_empty()
        );
    }

    #[test]
    fn combinations_within_prunes_far_spans() {
        // Points at 0 / 50 / 500 mm: with a 60 mm span limit only the
        // {0,1} pair can ever satisfy a small-figure fingerprint.
        let positions = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(500.0, 0.0),
        ];
        let pos_of = |i: usize| positions[i];
        let combos = combinations_within(&[0, 1, 2], 2, &pos_of, 60.0.into());
        assert_eq!(combos.len(), 1);
        assert_eq!(&combos[0][..], &[0, 1]);
    }

    #[test]
    fn combinations_within_degenerates_to_full_enumeration() {
        // A span limit beyond every pairwise distance keeps all
        // combinations - the worst case degrades to full enumeration.
        let positions = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(500.0, 0.0),
        ];
        let pos_of = |i: usize| positions[i];
        let combos = combinations_within(&[0, 1, 2], 2, &pos_of, 1000.0.into());
        assert_eq!(combos.len(), 3);
    }

    /// Soundness invariant (SPATIAL_CLUSTERING.md golden property):
    /// every pruned combination's pairwise diameter provably exceeds
    /// the footprint - pruning can never reject a matching candidate.
    #[test]
    fn pruned_combinations_provably_exceed_footprint() {
        let positions = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(80.0, 40.0),
            Vec2::new(500.0, 0.0),
            Vec2::new(520.0, 60.0),
            Vec2::new(900.0, 900.0),
        ];
        let pos_of = |i: usize| positions[i];
        let span = Millimeters::new(75.0);

        let full: Vec<_> = combinations(&[0, 1, 2, 3, 4, 5], 3)
            .iter()
            .cloned()
            .collect();
        let pruned = combinations_within(&[0, 1, 2, 3, 4, 5], 3, &pos_of, span);

        // Pruned output is a strict subset of full enumeration.
        assert!(pruned.len() < full.len());
        assert!(pruned.iter().all(|c| full.contains(c)));
        // Every dropped combination's diameter exceeds the span bound.
        for combo in full.iter().filter(|c| !pruned.contains(c)) {
            let diameter = combo
                .iter()
                .flat_map(|&a| combo.iter().map(move |&b| pos_of(a).distance(&pos_of(b))))
                .max_by(Millimeters::total_cmp)
                .unwrap_or_default();
            assert!(
                diameter > span,
                "pruned combo {combo:?} has diameter {diameter:?} <= {span:?}"
            );
        }
    }

    #[test]
    fn footprint_is_max_distance_plus_tolerance() {
        // Square 40 mm sides, 56.57 mm diagonals + 5 mm tolerance.
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 40.0),
        ];
        let pattern = TangiblePattern::from_points(1, "square", (&square).into(), 5.0);
        let footprint = pattern.footprint().as_f64();
        let expected = (40.0f64 * 2.0f64.sqrt()) + 5.0;
        assert!((footprint - expected).abs() < 0.01);
    }

    #[test]
    fn match_inside_footprint_still_finds_the_pattern() {
        // A candidate spanning right at the footprint boundary must
        // still match - pruning is exact, never an approximation.
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
        // 1.05x scaled: span 52.5*sqrt(2) = 74.25 <= footprint 75.7
        // (70.7 + 5) - the boundary candidate stays inside the prune.
        let touches = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 152.5, 100.0),
            touch_point(2, 100.0, 152.5),
            touch_point(3, 999.0, 999.0), // distractor, always pruned
        ];
        let matches = PatternSet::from(&patterns[..]).find_matches((&touches).into());
        assert_eq!(matches.len(), 1);
        assert_eq!(&matches[0].touch_indices[..], &[0, 1, 2]);
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
        let matches = PatternSet::from(&patterns[..]).find_matches_with_diagnostics(
            (&touches).into(),
            &mut reports,
            &TakenIndices::default(),
            &CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        // Span pruning (SPATIAL_CLUSTERING.md): combos containing the
        // (999,999) touch exceed the triangle footprint and are never
        // evaluated - diagnostics stay sparse: only the surviving
        // C(3,3) candidate reports.
        assert_eq!(reports.len(), 1);
        assert_eq!(reports.iter().filter(|report| report.matched).count(), 1);
        assert!(reports.iter().all(|report| !report.partial));

        let matched = reports.iter().find(|report| report.matched).expect("match");
        assert_eq!(matched.touch_indices, vec![0, 1, 2].into());
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
        let matches = PatternSet::from(&patterns[..]).find_matches_with_diagnostics(
            (&touches).into(),
            &mut reports,
            &TakenIndices::default(),
            &CompletionConfig::default(),
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
        let matches = PatternSet::from(&patterns[..]).find_matches_with_diagnostics(
            (&touches).into(),
            &mut reports,
            &TakenIndices::default(),
            &CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].pattern_name.as_str().ends_with("(partial)"));
        // Span pruning (SPATIAL_CLUSTERING.md): the exact {0,1,2,3}
        // combo and every partial containing the (999,999) touch exceed
        // the square footprint - only the surviving {0,1,2} partial
        // reports.
        assert_eq!(reports.len(), 1);
        let partials: Vec<_> = reports.iter().filter(|report| report.partial).collect();
        assert_eq!(partials.len(), 1);
        assert_eq!(partials.iter().filter(|report| report.matched).count(), 1);
        // A 3-pin subset has C(3,2) = 3 distances.
        assert!(partials.iter().all(|report| report.deviations.len() == 3));
    }

    #[test]
    fn region_rejects_a_geometrically_valid_match() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&reference).into(), 5.0);
        // Only the left half of the surface may match.
        pattern.regions = vec![Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(200.0),
            y_max: Millimeters::new(1000.0),
        }];
        let patterns = vec![pattern];

        let inside = vec![
            touch_point(0, 100.0, 100.0),
            touch_point(1, 150.0, 100.0),
            touch_point(2, 100.0, 150.0),
        ];
        assert_eq!(
            PatternSet::from(&patterns[..])
                .find_matches((&inside).into())
                .len(),
            1
        );

        // Same fingerprint on the right half: region rejects it.
        let outside = vec![
            touch_point(0, 400.0, 100.0),
            touch_point(1, 450.0, 100.0),
            touch_point(2, 400.0, 150.0),
        ];
        assert!(
            PatternSet::from(&patterns[..])
                .find_matches((&outside).into())
                .is_empty()
        );
    }

    #[test]
    fn region_boundary_point_rejects_the_candidate() {
        let reference = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let mut pattern = TangiblePattern::from_points(1, "triangle", (&reference).into(), 5.0);
        pattern.regions = vec![Region {
            x_min: Millimeters::new(0.0),
            y_min: Millimeters::new(0.0),
            x_max: Millimeters::new(200.0),
            y_max: Millimeters::new(1000.0),
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
            PatternSet::from(&patterns[..])
                .find_matches((&straddling).into())
                .len(),
            1
        );
        assert!(
            PatternSet::from(&patterns[..])
                .find_matches((&outside).into())
                .is_empty()
        );
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
        let matches = PatternSet::from(&patterns[..]).find_matches_with_diagnostics(
            (&touches).into(),
            &mut reports,
            &TakenIndices::default(),
            &CompletionConfig::default(),
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

        let matches = PatternSet::from(&patterns[..]).find_matches_excluding(
            (&touches).into(),
            &TakenIndices::default(),
            &CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].is_partial());
        // The near-predicted touch is claimed, the distant one is not.
        assert_eq!(matches[0].claimed_indices, vec![(3, 0).into()].into());
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

        let matches = PatternSet::from(&patterns[..]).find_matches_excluding(
            (&touches).into(),
            &TakenIndices::default(),
            &CompletionConfig::default(),
        );

        assert_eq!(matches.len(), 1);
        assert!(matches[0].is_partial());
        assert!(matches[0].claimed_indices.is_empty());
    }
}
