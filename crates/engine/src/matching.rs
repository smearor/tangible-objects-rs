use std::collections::HashSet;

use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::Vec2;

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

        let mut best: Option<(PatternMatch, Millimeters)> = None;

        for combo in combinations(&available, pattern.num_points) {
            let points: Vec<Vec2<Millimeters>> = combo.iter().map(|&i| touches[i].pos).collect();

            if pattern.matches(&points) {
                let error = pattern.match_error(&points);
                let candidate = build_match(pattern, &points, &combo, false);
                consider(&mut best, candidate, error);
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

                if let Some((_ref_indices, error)) = pattern.matches_partial(&points) {
                    let candidate = build_match(pattern, &points, &combo, true);
                    consider(&mut best, candidate, error);
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

/// Keeps the better of two candidates (lowest total error wins).
fn consider(
    best: &mut Option<(PatternMatch, Millimeters)>,
    candidate: PatternMatch,
    error: Millimeters,
) {
    match best {
        None => *best = Some((candidate, error)),
        Some((_, best_err)) if error < *best_err => *best = Some((candidate, error)),
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
}
