use std::collections::BTreeMap;
use std::fmt::Display;

use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;

use crate::Percentiles;
use crate::Recording;

/// Per-distance-index deviation statistics of one pattern over a
/// recording - the causal "why did the match fail" answer.
///
/// `index i` refers to the pattern's `sorted_distances[i]`, i.e. a
/// concrete pin pair; a high `blocker_frames` pinpoints the unstable
/// pin pair that most often pushed the deviation over tolerance.
#[derive(Debug, Serialize)]
pub struct DistanceStats {
    /// Distance index into the pattern's sorted fingerprint.
    pub index: usize,
    /// Frames this index was measured (full candidates only).
    pub samples: FrameCount,
    /// Mean absolute deviation in millimeters.
    pub mean: Millimeters,
    /// Worst observed deviation in millimeters.
    pub max: Millimeters,
    /// 99th percentile deviation in millimeters.
    pub p99: Millimeters,
    /// Frames where this index exceeded its effective tolerance
    /// (per-index `tolerances[i]` when configured, else the scalar).
    pub over_tolerance: FrameCount,
    /// Frames where this index was the **only** over-tolerance
    /// distance - the direct cause of the miss.
    pub sole_blocker: FrameCount,
    /// Frames where this index was among the over-tolerance
    /// distances.
    pub blocker: FrameCount,
}

/// Per-pattern deviation analysis.
#[derive(Debug, Serialize)]
pub struct PatternDeviation {
    /// Pattern the statistics belong to.
    pub pattern_id: PatternId,
    /// Profile name.
    pub pattern_name: PatternName,
    /// Frames with a matched full candidate.
    pub matched_frames: FrameCount,
    /// Frames with a full candidate that did not match.
    pub missed_frames: FrameCount,
    /// Per-distance-index statistics, in fingerprint order.
    pub distances: Vec<DistanceStats>,
}

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
    pub fn analyze(recording: &Recording) -> Self {
        struct PatternAcc {
            name: PatternName,
            matched: u64,
            missed: u64,
            deviations: Vec<Vec<f64>>,
            over: Vec<u64>,
            sole: Vec<u64>,
            blocker: Vec<u64>,
        }

        let mut acc: BTreeMap<PatternId, PatternAcc> = BTreeMap::new();

        for frame in recording.frames() {
            for candidate in &frame.candidates {
                if candidate.partial {
                    continue;
                }
                let entry = acc
                    .entry(candidate.pattern_id)
                    .or_insert_with(|| PatternAcc {
                        name: candidate.pattern_name.clone(),
                        matched: 0,
                        missed: 0,
                        deviations: Vec::new(),
                        over: Vec::new(),
                        sole: Vec::new(),
                        blocker: Vec::new(),
                    });
                if candidate.matched {
                    entry.matched += 1;
                } else {
                    entry.missed += 1;
                }

                // Effective tolerance per index: per-index tolerances
                // when configured, else the scalar fallback.
                let tolerances_at = |index: usize| {
                    candidate
                        .tolerances
                        .get(index)
                        .copied()
                        .unwrap_or_else(|| candidate.tolerance.as_millimeters())
                };

                let blockers: Vec<usize> = candidate
                    .deviations
                    .iter()
                    .enumerate()
                    .filter(|(index, deviation)| {
                        deviation.as_f64() > tolerances_at(*index).as_f64()
                    })
                    .map(|(index, _)| index)
                    .collect();

                for (index, deviation) in candidate.deviations.iter().enumerate() {
                    let over = deviation.as_f64() > tolerances_at(index).as_f64();
                    if entry.deviations.len() <= index {
                        entry.deviations.resize_with(index + 1, Vec::new);
                        entry.over.resize(index + 1, 0);
                        entry.sole.resize(index + 1, 0);
                        entry.blocker.resize(index + 1, 0);
                    }
                    entry.deviations[index].push(deviation.as_f64());
                    if over {
                        entry.over[index] += 1;
                    }
                    if blockers.len() == 1 && blockers[0] == index {
                        entry.sole[index] += 1;
                    }
                    if blockers.contains(&index) {
                        entry.blocker[index] += 1;
                    }
                }
            }
        }

        let patterns = acc
            .into_iter()
            .map(|(pattern_id, data)| PatternDeviation {
                pattern_id,
                pattern_name: data.name,
                matched_frames: FrameCount::new(data.matched),
                missed_frames: FrameCount::new(data.missed),
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
                            over_tolerance: FrameCount::new(data.over[index]),
                            sole_blocker: FrameCount::new(data.sole[index]),
                            blocker: FrameCount::new(data.blocker[index]),
                        })
                    })
                    .collect(),
            })
            .collect();

        Self { patterns }
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
