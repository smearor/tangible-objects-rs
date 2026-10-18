use std::collections::BTreeMap;
use std::fmt::Display;

use tangible_objects_model::CandidateCount;
use tangible_objects_model::ContactCount;
use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternName;

use super::LifetimeBucket;
use super::PartialShare;
use crate::Percentiles;
use crate::Recording;

/// Histogram bucket edges for contact lifetimes (in frames).
const LIFETIME_EDGES: [FrameCount; 5] = [
    FrameCount::new(1),
    FrameCount::new(5),
    FrameCount::new(15),
    FrameCount::new(60),
    FrameCount::new(120),
];

/// Contact-lifetime and dropout statistics of a recording (TRAINING.md 2.3).
///
/// Built from the `tracking_id` time series (`touches`) and the tracker's
/// `frames_since_match` hysteresis (`tracked`). Per-pin attribution is not
/// derivable from recordings - `touch_indices` is an unordered combination,
/// not a pin mapping - so instability is reported per contact, not per pin.
#[derive(Debug)]
pub struct FlappyReport {
    /// Number of distinct tracking IDs observed.
    pub contacts: usize,
    /// Percentiles of contact lifetimes in frames (first→last appearance).
    pub lifetimes: Option<Percentiles>,
    /// Lifetime histogram; the last bucket is open-ended.
    pub lifetime_buckets: Vec<LifetimeBucket>,
    /// Dropout events: runs of `frames_since_match > 0` inside a tracked
    /// tangible's lifetime - the figure lost its match but stayed tracked.
    pub dropout_events: u64,
    /// Percentiles of dropout run lengths in frames.
    pub dropout_lengths: Option<Percentiles>,
    /// Partial-match share per pattern.
    pub partial_share: Vec<PartialShare>,
}

impl FlappyReport {
    /// Computes the flappy-contact report for a loaded recording.
    pub fn analyze(recording: &Recording) -> Self {
        // Contact lifetimes from the tracking_id time series.
        let mut first_seen: BTreeMap<i32, u64> = BTreeMap::new();
        let mut lifetimes: BTreeMap<i32, u64> = BTreeMap::new();
        for frame in recording.frames() {
            for touch in &frame.touches {
                let id = touch.tracking_id.0;
                let first = first_seen.entry(id).or_insert(frame.frame.as_u64());
                lifetimes.insert(id, frame.frame.as_u64() - *first + 1);
            }
        }

        let lifetime_values: Vec<f64> = lifetimes.values().map(|v| *v as f64).collect();
        let mut lifetime_buckets: Vec<LifetimeBucket> = LIFETIME_EDGES
            .iter()
            .map(|edge| LifetimeBucket {
                upper_bound: Some(*edge),
                count: ContactCount::new(0),
            })
            .collect();
        lifetime_buckets.push(LifetimeBucket {
            upper_bound: None,
            count: ContactCount::new(0),
        });
        for &lifetime in lifetimes.values() {
            let index = LIFETIME_EDGES
                .iter()
                .position(|edge| lifetime <= edge.as_u64())
                .unwrap_or(LIFETIME_EDGES.len());
            lifetime_buckets[index].count += ContactCount::new(1);
        }

        // Dropout runs inside each tracked tangible's lifetime.
        let mut dropout_events = 0u64;
        let mut dropout_lengths = Vec::new();
        let mut runs: BTreeMap<u32, u64> = BTreeMap::new();
        for frame in recording.frames() {
            for tracked in &frame.tracked {
                if tracked.frames_since_match > 0 {
                    *runs.entry(tracked.id.0).or_insert(0) += 1;
                } else if let Some(length) = runs.remove(&tracked.id.0) {
                    dropout_events += 1;
                    dropout_lengths.push(length as f64);
                }
            }
        }
        for (_, length) in runs {
            dropout_events += 1;
            dropout_lengths.push(length as f64);
        }

        // Partial-match share per pattern.
        let mut share: BTreeMap<PatternName, PartialShare> = BTreeMap::new();
        for frame in recording.frames() {
            for candidate in &frame.candidates {
                if !candidate.matched {
                    continue;
                }
                let entry = share
                    .entry(candidate.pattern_name.clone())
                    .or_insert_with(|| PartialShare {
                        pattern_name: candidate.pattern_name.clone(),
                        partial_matches: CandidateCount::new(0),
                        matched_total: CandidateCount::new(0),
                    });
                entry.matched_total += 1;
                if candidate.partial {
                    entry.partial_matches += 1;
                }
            }
        }

        Self {
            contacts: lifetimes.len(),
            lifetimes: Percentiles::of(lifetime_values.iter().copied()),
            lifetime_buckets,
            dropout_events,
            dropout_lengths: Percentiles::of(dropout_lengths.iter().copied()),
            partial_share: share.into_values().collect(),
        }
    }
}

impl Display for FlappyReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "== Flappy contacts == ({} contacts)", self.contacts)?;
        if let Some(lifetimes) = self.lifetimes {
            writeln!(
                f,
                "  lifetime frames: p50={:.0} p95={:.0} p99={:.0} max={:.0} (mean {:.1})",
                lifetimes.p50, lifetimes.p95, lifetimes.p99, lifetimes.max, lifetimes.mean
            )?;
        }
        for bucket in &self.lifetime_buckets {
            let label = match bucket.upper_bound {
                Some(edge) => format!("<= {edge}"),
                None => format!("> {}", LIFETIME_EDGES[LIFETIME_EDGES.len() - 1]),
            };
            writeln!(f, "    {label}: {}", bucket.count)?;
        }
        writeln!(f, "  dropout events: {}", self.dropout_events)?;
        if let Some(lengths) = self.dropout_lengths {
            writeln!(
                f,
                "    lengths: p50={:.0} p95={:.0} max={:.0}",
                lengths.p50, lengths.p95, lengths.max
            )?;
        }
        for share in &self.partial_share {
            writeln!(
                f,
                "  {}: {}/{} partial matches ({:.1}%)",
                share.pattern_name,
                share.partial_matches,
                share.matched_total,
                share.fraction() * 100.0
            )?;
        }
        Ok(())
    }
}
