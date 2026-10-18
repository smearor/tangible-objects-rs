use std::collections::BTreeMap;
use std::fmt::Display;

use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternId;
use tangible_objects_model::PointSet;
use tangible_objects_model::TangiblePattern;

use super::DistanceStats;
use super::PatternAcc;
use super::PatternDeviation;
use crate::Percentiles;
use crate::Recording;

/// Automated deviation detail analysis over a recording.
///
/// Folds every *full* candidate (matched or rejected) of every pattern
/// into per-index statistics: mean/p99/max deviation, over-tolerance
/// counts and the "blocker" histogram answering which pin pair most
/// often killed the match. Equivalent to manually inspecting
/// `candidates[].deviations` around dropout windows - but over the
/// whole recording and per pattern.
#[derive(Debug, Serialize)]
pub struct DeviationReport {
    /// One analysis per pattern that produced at least one full
    /// candidate; sorted by pattern ID.
    pub patterns: Vec<PatternDeviation>,
}

impl DeviationReport {
    /// Folds the recording into per-pattern per-index statistics.
    ///
    /// Covers only *full generic candidates* - frames whose contacts run
    /// through the incumbent path produce no candidates and are invisible
    /// here; see [`Self::analyze_contacts`] for the contact-level pass.
    pub fn analyze(recording: &Recording) -> Self {
        let mut acc: BTreeMap<PatternId, PatternAcc> = BTreeMap::new();

        for frame in recording.frames() {
            for candidate in &frame.candidates {
                if candidate.partial {
                    continue;
                }
                let entry = acc
                    .entry(candidate.pattern_id)
                    .or_insert_with(|| PatternAcc::new(candidate.pattern_name.clone()));

                // Effective tolerance per index: per-index tolerances
                // when configured, else the scalar fallback.
                let tolerance_at = |index: usize| {
                    candidate
                        .tolerances
                        .get(index)
                        .copied()
                        .unwrap_or_else(|| candidate.tolerance.as_millimeters())
                };
                entry.push(&candidate.deviations, candidate.matched, tolerance_at);
            }
        }

        Self {
            patterns: acc
                .into_iter()
                .map(|(pattern_id, data)| Self::finish(pattern_id, data))
                .collect(),
        }
    }

    /// Folds every recorded *contact set* into per-index statistics:
    /// for each frame and pattern all `num_points`-sized subsets of
    /// `frame.touches` are evaluated against the pattern fingerprint,
    /// regardless of whether the frame produced a generic candidate.
    /// Covers the incumbent-tracked majority of frames that
    /// [`Self::analyze`] cannot see.
    pub fn analyze_contacts(recording: &Recording, patterns: &[TangiblePattern]) -> Self {
        let mut acc: BTreeMap<PatternId, PatternAcc> = BTreeMap::new();

        for frame in recording.frames() {
            let touches: Vec<_> = frame.touches.iter().map(|point| point.pos).collect();
            for pattern in patterns {
                // Only the best-fitting subset counts per frame - foreign
                // contacts (hands, other objects) would otherwise inflate
                // the deviation statistics with non-figure geometry.
                let mut best: Option<(Vec<Millimeters>, f64)> = None;
                for subset in Self::combinations(touches.len(), pattern.num_points.as_usize()) {
                    let points: Vec<_> = subset.iter().map(|&index| touches[index]).collect();
                    let deviations = PointSet::new(&points)
                        .distances()
                        .deviations(&pattern.sorted_distances);
                    let score = pattern.score_of(&deviations);
                    if best.as_ref().is_none_or(|(_, s)| score < *s) {
                        best = Some((deviations, score));
                    }
                }
                if let Some((deviations, score)) = best {
                    let entry = acc
                        .entry(pattern.id)
                        .or_insert_with(|| PatternAcc::new(pattern.name.clone()));
                    entry.push(&deviations, score <= 1.0, |index| {
                        pattern.tolerance_at(index).as_millimeters()
                    });
                }
            }
        }

        Self {
            patterns: acc
                .into_iter()
                .map(|(pattern_id, data)| Self::finish(pattern_id, data))
                .collect(),
        }
    }

    /// All k-element index subsets of `n` items, in lexicographic
    /// order - used by `analyze_contacts` to enumerate same-size
    /// contact subsets per frame.
    fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
        if k == 0 || k > n {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut current = Vec::with_capacity(k);
        fn visit(
            start: usize,
            n: usize,
            k: usize,
            current: &mut Vec<usize>,
            result: &mut Vec<Vec<usize>>,
        ) {
            if current.len() == k {
                result.push(current.clone());
                return;
            }
            for index in start..=n - (k - current.len()) {
                current.push(index);
                visit(index + 1, n, k, current, result);
                current.pop();
            }
        }
        visit(0, n, k, &mut current, &mut result);
        result
    }

    fn finish(pattern_id: PatternId, data: PatternAcc) -> PatternDeviation {
        PatternDeviation {
            pattern_id,
            pattern_name: data.name,
            matched_frames: data.matched,
            missed_frames: data.missed,
            distances: data
                .deviations
                .into_iter()
                .enumerate()
                .filter_map(|(index, samples)| {
                    let stats = Percentiles::of(samples.iter().copied())?;
                    Some(DistanceStats {
                        index,
                        samples: FrameCount::new(samples.len() as u64),
                        mean: Millimeters::new(stats.mean),
                        max: Millimeters::new(stats.max),
                        p99: Millimeters::new(stats.p99),
                        over_tolerance: data.over[index],
                        sole_blocker: data.sole[index],
                        blocker: data.blocker[index],
                    })
                })
                .collect(),
        }
    }
}

impl Display for DeviationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Abweichungs-Analyse (welcher Abstand blockiert) ===")?;
        for pattern in &self.patterns {
            writeln!(
                f,
                "{} (id {}): {} matched / {} missed",
                pattern.pattern_name,
                pattern.pattern_id,
                pattern.matched_frames,
                pattern.missed_frames
            )?;
            for stats in &pattern.distances {
                writeln!(
                    f,
                    "  d[{index}]: mean={mean:.2} max={max:.2} p99={p99:.2} \
                     | over-tol={over} blocker={blocker} sole={sole}",
                    index = stats.index,
                    mean = stats.mean.as_f64(),
                    max = stats.max.as_f64(),
                    p99 = stats.p99.as_f64(),
                    over = stats.over_tolerance,
                    blocker = stats.blocker,
                    sole = stats.sole_blocker,
                )?;
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::DeviationReport;
    use crate::Recording;

    /// Writes a one-frame recording whose candidate over-tolerates only
    /// `deviations[1]` - the report must flag index 1 as sole blocker.
    #[test]
    fn flags_the_blocking_distance() {
        let path = std::env::temp_dir().join("deviation-test.jsonl");
        let mut file = std::fs::File::create(&path).expect("create");
        let candidate = r#"{"pattern_id":5,"pattern_name":"fig","touch_indices":[0,1,2],"matched":false,"partial":false,"measured_distances":[30.0,40.0,50.0],"deviations":[1.0,4.5,0.5],"tolerance":3.0,"match_error":6.0}"#;
        for frame in [1, 2, 3] {
            let line = format!(
                r#"{{"v":1,"frame":{frame},"timestamp_ms":{frame},"label":null,"calibration":null,"touches":[],"candidates":[{candidate}],"tracked":[],"raw_dropped":0}}"#
            );
            writeln!(file, "{line}").expect("write");
        }
        drop(file);

        let recording = Recording::load(&path).expect("load");
        std::fs::remove_file(&path).ok();

        let report = DeviationReport::analyze(&recording);
        assert_eq!(report.patterns.len(), 1);
        let pattern = &report.patterns[0];
        assert_eq!(pattern.missed_frames.as_u64(), 3);
        assert_eq!(pattern.distances.len(), 3);
        assert_eq!(pattern.distances[1].over_tolerance.as_u64(), 3);
        assert_eq!(pattern.distances[1].sole_blocker.as_u64(), 3);
        assert_eq!(pattern.distances[1].blocker.as_u64(), 3);
        assert_eq!(pattern.distances[0].blocker.as_u64(), 0);
        assert_eq!(pattern.distances[2].blocker.as_u64(), 0);
        assert!((pattern.distances[1].max.as_f64() - 4.5).abs() < 1e-6);
        assert!((pattern.distances[1].mean.as_f64() - 4.5).abs() < 1e-6);
    }
}
