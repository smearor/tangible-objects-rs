use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Display;

use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternId;

use super::Accumulator;
use super::PatternUtilization;
use crate::Percentiles;
use crate::Recording;

/// Tolerance-utilization report over a whole recording (TRAINING.md 2.1).
#[derive(Debug)]
pub struct UtilizationReport {
    /// Total recorded frames.
    pub total_frames: FrameCount,
    /// Frames carrying a ground-truth label.
    pub labeled_frames: FrameCount,
    /// Per-pattern statistics, sorted by pattern ID (label-only rows last).
    pub patterns: Vec<PatternUtilization>,
}

impl UtilizationReport {
    /// Computes the utilization report for a loaded recording.
    ///
    /// Deviation statistics come from the labeled pattern's best candidate
    /// per labeled frame - matched or not. Sampling only `matched`
    /// candidates would bias percentiles toward the configured tolerance
    /// (survivorship: worse misses never enter the statistics).
    pub fn analyze(recording: &Recording) -> Self {
        let mut labeled_frames = 0u64;
        let mut acc: BTreeMap<PatternId, (String, Accumulator)> = BTreeMap::new();
        // Label-only rows: labels that never matched a candidate's pattern.
        let mut label_only: BTreeMap<String, Accumulator> = BTreeMap::new();

        for frame in recording.frames() {
            if frame.label.is_some() {
                labeled_frames += 1;
            }

            let mut matched_full: BTreeSet<PatternId> = BTreeSet::new();
            let mut matched_partial: BTreeSet<PatternId> = BTreeSet::new();
            for candidate in &frame.candidates {
                if !candidate.matched {
                    continue;
                }
                if candidate.partial {
                    matched_partial.insert(candidate.pattern_id);
                } else {
                    matched_full.insert(candidate.pattern_id);
                    acc.entry(candidate.pattern_id)
                        .or_insert_with(|| {
                            (
                                candidate.pattern_name.as_str().to_string(),
                                Accumulator::default(),
                            )
                        })
                        .1
                        .matched_frames += 1;
                }
            }

            // Deviation statistics: the labeled pattern's best candidate
            // per labeled frame - matched or not. Sampling only `matched`
            // candidates biases percentiles toward the configured tolerance
            // (survivorship: worse misses never enter the statistics).
            if let Some(label) = &frame.label {
                for candidate in frame.best_candidates().values() {
                    if candidate.partial || !candidate.matches_label(label) {
                        continue;
                    }
                    let entry = &mut acc
                        .entry(candidate.pattern_id)
                        .or_insert_with(|| {
                            (
                                candidate.pattern_name.as_str().to_string(),
                                Accumulator::default(),
                            )
                        })
                        .1;
                    for (index, deviation) in candidate.deviations.iter().enumerate() {
                        if entry.deviations.len() <= index {
                            entry.deviations.resize_with(index + 1, Vec::new);
                        }
                        entry.deviations[index].push(*deviation);
                    }
                    // The score is already the per-index normalized
                    // deviation (max_i dev_i / tolerance_at(i)) - using the
                    // recorded scalar tolerance would ignore `tolerances[]`.
                    let ratio = candidate.normalized_score().as_f64();
                    if ratio >= 1.0 {
                        entry.over_tolerance_frames += 1;
                    } else {
                        let band = if ratio < 0.25 {
                            0
                        } else if ratio < 0.5 {
                            1
                        } else if ratio < 0.75 {
                            2
                        } else {
                            3
                        };
                        entry.bands[band] += 1;
                    }
                }
            }

            for pattern_id in matched_partial.difference(&matched_full) {
                acc.entry(*pattern_id)
                    .or_insert_with(|| {
                        let name = frame
                            .candidates
                            .iter()
                            .find(|c| c.pattern_id == *pattern_id)
                            .map(|c| c.pattern_name.as_str().to_string())
                            .unwrap_or_default();
                        (name, Accumulator::default())
                    })
                    .1
                    .partial_matched_frames += 1;
            }

            // False negatives: labeled figure present but not detected.
            if let Some(label) = &frame.label {
                let mut label_seen = false;
                for candidate in &frame.candidates {
                    if !candidate.matches_label(label) {
                        continue;
                    }
                    label_seen = true;
                    let entry = &mut acc
                        .entry(candidate.pattern_id)
                        .or_insert_with(|| {
                            (
                                candidate.pattern_name.as_str().to_string(),
                                Accumulator::default(),
                            )
                        })
                        .1;
                    entry.labeled_frames += 1;
                    if !matched_full.contains(&candidate.pattern_id)
                        && !matched_partial.contains(&candidate.pattern_id)
                    {
                        entry.false_negative_frames += 1;
                    }
                }
                // The labeled pattern produced no candidate at all (fewer
                // touches than pins) - still a false negative; numeric labels
                // merge into the pattern's row, names land in label_only.
                if !label_seen {
                    if let Ok(id) = label.as_str().parse::<u32>() {
                        acc.entry(PatternId::new(id))
                            .or_insert_with(|| (label.as_str().to_string(), Accumulator::default()))
                            .1
                            .labeled_frames += 1;
                        acc.get_mut(&PatternId::new(id))
                            .expect("inserted")
                            .1
                            .false_negative_frames += 1;
                    } else {
                        let entry = label_only.entry(label.as_str().to_string()).or_default();
                        entry.labeled_frames += 1;
                        entry.false_negative_frames += 1;
                    }
                }
            }
        }

        let mut patterns: Vec<PatternUtilization> = acc
            .into_iter()
            .map(|(pattern_id, (pattern_name, entry))| {
                PatternUtilization::from((Some(pattern_id), pattern_name, entry))
            })
            .collect();
        patterns.extend(
            label_only
                .into_iter()
                .map(|(label, entry)| PatternUtilization::from((None, label, entry))),
        );

        Self {
            total_frames: FrameCount::from(recording.frames().len()),
            labeled_frames: FrameCount::new(labeled_frames),
            patterns,
        }
    }
}

impl From<(Option<PatternId>, String, Accumulator)> for PatternUtilization {
    fn from((pattern_id, pattern_name, entry): (Option<PatternId>, String, Accumulator)) -> Self {
        Self {
            pattern_id,
            pattern_name,
            matched_frames: entry.matched_frames,
            partial_matched_frames: entry.partial_matched_frames,
            labeled_frames: entry.labeled_frames,
            false_negative_frames: entry.false_negative_frames,
            per_distance: entry
                .deviations
                .iter()
                .filter_map(|samples| Percentiles::of(samples.iter().copied()))
                .collect(),
            bands: entry.bands,
            over_tolerance_frames: entry.over_tolerance_frames,
        }
    }
}

impl Display for UtilizationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "== Utilization == ({} frames, {} labeled)",
            self.total_frames, self.labeled_frames
        )?;
        for pattern in &self.patterns {
            let id = pattern
                .pattern_id
                .map(|id| id.0.to_string())
                .unwrap_or_else(|| "?".to_string());
            writeln!(
                f,
                "  {} (id={}): {} matched (+{} partial), {} labeled, {} false negatives",
                pattern.pattern_name,
                id,
                pattern.matched_frames,
                pattern.partial_matched_frames,
                pattern.labeled_frames,
                pattern.false_negative_frames
            )?;
            for (index, stats) in pattern.per_distance.iter().enumerate() {
                writeln!(
                    f,
                    "    dist[{index}]: p50={:.2} p95={:.2} p99={:.2} max={:.2} (n={})",
                    stats.p50, stats.p95, stats.p99, stats.max, stats.count
                )?;
            }
            writeln!(
                f,
                "    bands: <25%={} | 25-50%={} | 50-75%={} | 75-100%={} | >100% (missed)={}",
                pattern.bands[0],
                pattern.bands[1],
                pattern.bands[2],
                pattern.bands[3],
                pattern.over_tolerance_frames
            )?;
        }
        Ok(())
    }
}
